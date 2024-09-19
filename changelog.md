# Changelog

## [0.1.5] - 2024-09-19
- **Added**: Functions `fmt_file` and `lint_file` for formatting and linting Rust files.
  - `fmt_file` uses `rustfmt` to format the specified Rust file.
  - `lint_file` uses `cargo clippy` to perform linting on the specified Rust file.