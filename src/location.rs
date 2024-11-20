use std::{
    fmt::Display,
    // fs,
    path::{Path, PathBuf},
};

/// Represents a source location in a file system
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawLoc {
    pub filename: PathBuf,
    pub lines: Vec<u32>, // Line numbers as u32
}

impl RawLoc {
    /// Create a new `RawLoc` from a file path and line numbers
    pub fn new(filename: PathBuf, lines: Vec<u32>) -> Self {
        Self { filename, lines }
    }
}

/// Represents a location with a path and function name
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Loc(PathBuf, String);

impl Loc {
    /// Get the path associated with the location
    pub fn path(&self) -> &PathBuf {
        &self.0
    }

    /// Get the function name from the location
    pub fn fn_name(&self) -> &str {
        self.1.split("::").last().unwrap_or("")
    }

    /// Get the full function name from the location
    pub fn full_fn_name(&self) -> &str {
        &self.1
    }

    /// Get the file name from the full function name
    pub fn file_name(&self) -> String {
        let split_str = self.1.split("::");
        let mut split_as_vec = split_str.collect::<Vec<&str>>();
        split_as_vec.pop();
        split_as_vec.join("::")
    }

    /// Read the source code from the file system
    pub fn read_source<S: FileSystem>(&self, fs: &S) -> Result<String, S::FSError> {
        fs.read(&self.0)
    }

    /// Write the source code to the file system
    pub fn write_source<S: FileSystem>(&self, fs: &S, str: &str) -> Result<(), S::FSError> {
        fs.write(&self.0, str.as_bytes())
    }
}

impl Display for Loc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.0.to_str().unwrap_or(""), self.1)
    }
}

impl From<(RawLoc, String)> for Loc {
    fn from((loc, name): (RawLoc, String)) -> Self {
        Loc(loc.filename, name)
    }
}

/// Trait representing a file system abstraction
pub trait FileSystem: Clone {
    type FSError: std::fmt::Debug;

    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool, Self::FSError>;
    fn read<P: AsRef<Path>>(&self, filename: P) -> Result<String, Self::FSError>;
    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        filename: P,
        contents: C,
    ) -> Result<(), Self::FSError>;
}