<<<<<<< HEAD
# rem-utils

rem-utils is a utilities library for the Rusty Extraction Maestro (REM)
toolchain.

It is used as a reference for the following crates:

    - rem-controller
    - rem-borrower
    - rem-repairer
    - rem-constraint

## Requirements

It requires internal access to the rust toolchain, using
`#![feature(rustc_private)]`.

The rust-toolchain.toml file specifies the rest of the build dependecies, and is
shared across all of the REM toolchain.

As a minimum you should have:

    - rust-src
    - rust-dev
    - llvm-tools-preview

It is currently configured to run on the `nightly-2024-08-28` build of rust,
however, other nightly builds may also work.

## Installation

You can install these components by running:

    ```bash
    rustup component add --toolchain nightly-2024-08-28 rust-src rustc-dev llvm-tools-preview
    ```
=======
# rem-utils

rem-utils is a utilities library for the Rusty Extraction Maestro (REM)
toolchain.

It is used as a reference for the following crates:

    - rem-controller
    - rem-borrower
    - rem-repairer
    - rem-constraint

## Requirements

**As of version 0.1.4, this library is no longer dependent on rustc!**

The rust-toolchain.toml file specifies the rest of the build dependecies, and is shared across all of the REM toolchain.

As a minimum you should have:

    - rust-src
    - rust-dev
    - llvm-tools-preview

It is currently configured to run on the `nightly-2024-08-28` build of rust,
however, other nightly builds may also work.

## Installation

You can install these components by running:

```bash
rustup component add --toolchain nightly-2024-08-28 rust-src rustc-dev llvm-tools-preview
```

## Function Exports

1. **Compilation Utilities**
   - `compile_file(file_name: &str, args: &Vec<&str>) -> Command`: Compiles a Rust file using `rustc` with optional arguments.

2. **Formatting Utilities**
   - `fmt_file(file_name: &str, args: &Vec<&str>) -> Command`: Formats a Rust file using `rustfmt` with optional arguments.

3. **Linting Utilities**
   - `lint_file(file_name: &str, args: &Vec<&str>) -> Command`: Lints a Rust file using `cargo clippy` with optional arguments.

4. **Project Checking and Building**
   - `check_project(manifest_path: &str, cargo_args: &Vec<&str>) -> Command`: Checks the Rust project for errors using `cargo check`, with customizable arguments.
   - `build_project(manifest_path: &str, cargo_args: &Vec<&str>) -> Command`: Builds the Rust project using `cargo build`, with customizable arguments.

5. **Code Analysis**
   - `find_caller(file_name: &str, caller_name: &str, callee_name: &str, callee_body_only: bool) -> (bool, String, String)`: Finds a function call within the specified file and retrieves the caller and callee function definitions.

6. **Source Formatting**
   - `format_source(src: &str) -> String`: Formats Rust source code using `rustfmt`.
>>>>>>> a1b7036e19119c62935e9f7ecc72ecee3c33a837
