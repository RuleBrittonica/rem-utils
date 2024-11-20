use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::Display,
    fs,
    io::{self},
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

/// Representation of a file system
pub trait FileSystem: Clone {
    /// Errors
    type FSError: std::fmt::Debug;

    /// Tests for the existence of a given file
    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Self::FSError>;

    /// Reads a file, specified by a path into a string
    fn read<P: AsRef<Path>>(&self, filename: P) -> Result<String, Self::FSError>;

    /// Writes into a file specified by a path
    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        filename: P,
        contents: C,
    ) -> Result<(), Self::FSError>;
}

/// Wrapper over the underlying file system
#[derive(Copy, Clone)]
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    type FSError = io::Error;

    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Self::FSError> {
        Ok(path.as_ref().exists())
    }

    fn read<P: AsRef<Path>>(&self, filename: P) -> Result<String, Self::FSError> {
        fs::read_to_string(filename)
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        filename: P,
        contents: C,
    ) -> Result<(), Self::FSError> {
        fs::write(filename, contents)
    }
}

#[derive(Clone)]
pub struct SymbolicFileSystem(Rc<RefCell<HashMap<String, String>>>);

impl FileSystem for SymbolicFileSystem {
    type FSError = ();

    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Self::FSError> {
        let path_str = path.as_ref().to_str().unwrap_or("").to_string();
        Ok(self.0.borrow().contains_key(&path_str))
    }

    fn read<P: AsRef<Path>>(&self, filename: P) -> Result<String, Self::FSError> {
        let path_str = filename.as_ref().to_str().unwrap_or("").to_string();
        Ok(self.0.borrow().get(&path_str).cloned().unwrap_or_default())
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        filename: P,
        contents: C,
    ) -> Result<(), Self::FSError> {
        let path_str = filename.as_ref().to_str().unwrap_or("").to_string();
        let content_str = String::from_utf8_lossy(contents.as_ref()).to_string();
        self.0.borrow_mut().insert(path_str, content_str);
        Ok(())
    }
}

impl Display for SymbolicFileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SymbolicFileSystem {{")?;
        for (key, value) in self.0.borrow().iter() {
            writeln!(f, "file \"{}\": {{|", key)?;
            writeln!(f, "{}", value)?;
            writeln!(f, "|}}")?;
        }
        writeln!(f, "}}")
    }
}

impl SymbolicFileSystem {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let mut map = HashMap::new();
        let mut to_visit = vec![path.as_ref().to_path_buf()];

        while let Some(current_path) = to_visit.pop() {
            let metadata = fs::metadata(&current_path)?;
            if metadata.is_dir() {
                let dir_entries = fs::read_dir(&current_path)?;
                for entry in dir_entries {
                    let entry = entry?;
                    to_visit.push(entry.path());
                }
            } else {
                let contents = fs::read_to_string(&current_path)?;
                let canonical_path = fs::canonicalize(&current_path)?.to_str().unwrap().to_string();
                map.insert(canonical_path, contents);
            }
        }

        Ok(SymbolicFileSystem(Rc::new(RefCell::new(map))))
    }

    pub fn get(&self, path: &str) -> String {
        let path_buf = PathBuf::from(path);
        let canonical_path = fs::canonicalize(&path_buf)
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .unwrap_or_else(|| path.to_string());
        self.0.borrow().get(&canonical_path).cloned().unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct FileLoader<T: FileSystem>(T);

unsafe impl<T: FileSystem> Send for FileLoader<T> {}
unsafe impl<T: FileSystem> Sync for FileLoader<T> {}

impl<T: FileSystem> FileLoader<T> {
    pub fn new(fs: T) -> Self {
        FileLoader(fs)
    }

    pub fn file_exists(&self, path: &Path) -> bool {
        self.0.exists(path).unwrap_or(false)
    }

    pub fn read_file(&self, path: &Path) -> io::Result<String> {
        self.0.read(path).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))
    }

    pub fn read_binary_file(&self, path: &Path) -> io::Result<Arc<[u8]>> {
        let content = self.read_file(path)?;
        Ok(Arc::from(content.into_bytes().as_slice()))
    }
}