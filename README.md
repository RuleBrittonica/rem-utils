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
