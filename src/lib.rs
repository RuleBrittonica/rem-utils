#![feature(box_patterns)]
#![feature(never_type)]
#![feature(iter_intersperse)]

// extern crate rustc_driver;
// pub extern crate rustc_lint;
// pub extern crate rustc_span;
pub extern crate string_cache;

pub mod annotation;
pub mod error;
pub mod filesystem;
pub mod formatter;
pub mod labelling;
pub mod location;
pub mod macros;
pub mod parser;
pub mod typ;
pub mod wrappers;

use log::{debug, info};
use quote::ToTokens;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use syn::visit_mut::VisitMut;
use syn::{ExprCall, ExprMethodCall, File, ImplItemMethod, ItemFn, TraitItemMethod, parse_file};

use std::path::PathBuf;

use home::cargo_home;
use regex::Regex;
use colored::*;

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////        COMPILE        /////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn compile_file(file_name: &str, args: &Vec<&str>) -> Command {
    let mut compile = Command::new("rustc");
    for arg in args {
        compile.arg(arg);
    }
    compile.arg(file_name);
    compile
}

pub fn check_project(manifest_path: &str, cargo_args: &Vec<&str>) -> Command {
    let mut check = Command::new("cargo");
    check.arg("check");
    for arg in cargo_args {
        check.arg(arg);
    }
    let toml = format!("--manifest-path={}", manifest_path);
    check.arg(toml);
    check.arg("--message-format=json");
    check
}

pub fn build_project(manifest_path: &str, cargo_args: &Vec<&str>) -> Command {
    let mut check = Command::new("cargo");
    check.arg("build");
    for arg in cargo_args {
        check.arg(arg);
    }
    let toml = format!("--manifest-path={}", manifest_path);
    check.arg(toml);
    check.arg("--message-format=json");
    check
}

pub struct FindCallee<'a> {
    pub found: bool,
    pub callee_fn_name: &'a str,
}

impl VisitMut for FindCallee<'_> {
    fn visit_expr_call_mut(&mut self, i: &mut ExprCall) {
        let callee = i.func.as_ref().into_token_stream().to_string();
        debug!("looking at callee: {}", callee);
        match callee.contains(self.callee_fn_name) {
            true => self.found = true,
            false => syn::visit_mut::visit_expr_call_mut(self, i),
        }
    }

    fn visit_expr_method_call_mut(&mut self, i: &mut ExprMethodCall) {
        let callee = i.method.clone().into_token_stream().to_string();
        debug!("looking at callee: {}", callee);
        match callee.contains(self.callee_fn_name) {
            true => self.found = true,
            false => syn::visit_mut::visit_expr_method_call_mut(self, i),
        }
    }
}

pub struct FindCaller<'a> {
    caller_fn_name: &'a str,
    callee_finder: &'a mut FindCallee<'a>,
    found: bool,
    caller: String,
}

impl VisitMut for FindCaller<'_> {
    fn visit_impl_item_method_mut(&mut self, i: &mut ImplItemMethod) {
        if self.found {
            return;
        }
        debug!("{:?}", i);
        let id = i.sig.ident.to_string();
        match id == self.caller_fn_name {
            true => {
                self.callee_finder.visit_impl_item_method_mut(i);
                if !self.callee_finder.found {
                    return;
                }
                self.found = true;
                self.caller = i.into_token_stream().to_string();
            }
            false => {}
        }
        syn::visit_mut::visit_impl_item_method_mut(self, i);
    }

    fn visit_trait_item_method_mut(&mut self, i: &mut TraitItemMethod) {
        if self.found {
            return;
        }
        debug!("{:?}", i);
        let id = i.sig.ident.to_string();
        match id == self.caller_fn_name {
            true => {
                self.callee_finder.visit_trait_item_method_mut(i);
                if !self.callee_finder.found {
                    return;
                }
                self.found = true;
                self.caller = i.into_token_stream().to_string();
            }
            false => {}
        }
        syn::visit_mut::visit_trait_item_method_mut(self, i);
    }

    fn visit_item_fn_mut(&mut self, i: &mut ItemFn) {
        if self.found {
            return;
        }
        debug!("{:?}", i);
        let id = i.sig.ident.to_string();
        match id == self.caller_fn_name {
            true => {
                self.callee_finder.visit_item_fn_mut(i);
                if !self.callee_finder.found {
                    return;
                }
                self.found = true;
                self.caller = i.into_token_stream().to_string();
            }
            false => (),
        }
    }
}

pub struct FindFn<'a> {
    fn_name: &'a str,
    found: bool,
    fn_txt: String,
    body_only: bool,
}

impl VisitMut for FindFn<'_> {
    fn visit_impl_item_method_mut(&mut self, i: &mut ImplItemMethod) {
        if self.found {
            return;
        }
        debug!("{:?}", i);
        let id = i.sig.ident.to_string();
        match id == self.fn_name {
            true => {
                self.found = true;
                i.attrs = vec![];
                if self.body_only {
                    self.fn_txt = i.block.clone().into_token_stream().to_string();
                } else {
                    self.fn_txt = i.into_token_stream().to_string();
                }
            }
            false => {}
        }
        syn::visit_mut::visit_impl_item_method_mut(self, i);
    }

    fn visit_item_fn_mut(&mut self, i: &mut ItemFn) {
        if self.found {
            return;
        }
        debug!("{:?}", i);
        let id = i.sig.ident.to_string();
        match id == self.fn_name {
            true => {
                self.found = true;
                i.attrs = vec![];
                if self.body_only {
                    self.fn_txt = i.block.clone().into_token_stream().to_string();
                } else {
                    self.fn_txt = i.into_token_stream().to_string();
                }
            }
            false => (),
        }
    }

    fn visit_trait_item_method_mut(&mut self, i: &mut TraitItemMethod) {
        if self.found {
            return;
        }
        debug!("{:?}", i);
        let id = i.sig.ident.to_string();
        match id == self.fn_name {
            true => {
                self.found = true;
                i.attrs = vec![];
                if self.body_only {
                    self.fn_txt = "{}".to_string();
                } else {
                    self.fn_txt = i.into_token_stream().to_string();
                }
            }
            false => {}
        }
        syn::visit_mut::visit_trait_item_method_mut(self, i);
    }
}

pub fn find_caller(
    file_name: &str,
    caller_name: &str,
    callee_name: &str,
    callee_body_only: bool,
) -> (bool, String, String) {
    let file_content: String = fs::read_to_string(&file_name).unwrap().parse().unwrap();
    let mut file = syn::parse_str::<File>(file_content.as_str())
        .map_err(|e| format!("{:?}", e))
        .unwrap();

    let mut visit = FindCaller {
        caller_fn_name: caller_name,
        callee_finder: &mut FindCallee {
            found: false,
            callee_fn_name: callee_name,
        },
        found: false,
        caller: String::new(),
    };
    visit.visit_file_mut(&mut file);

    let mut callee = FindFn {
        fn_name: callee_name,
        found: false,
        fn_txt: String::new(),
        body_only: callee_body_only,
    };

    callee.visit_file_mut(&mut file);

    (
        visit.found && callee.found,
        format_source(visit.caller.as_str()),
        format_source(callee.fn_txt.as_str()),
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////          MISC          ////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn format_source(src: &str) -> String {
    let rustfmt = {
        let rustfmt_path = format!("{}/bin/rustfmt", cargo_home().unwrap().to_string_lossy());
        println!("{}", &rustfmt_path);
        let mut proc = Command::new(&rustfmt_path)
            .arg("--edition=2021")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = proc.stdin.take().unwrap();
        stdin.write_all(src.as_bytes()).unwrap();
        proc
    };

    let stdout = rustfmt.wait_with_output().unwrap();

    String::from_utf8(stdout.stdout).unwrap()
}

pub fn remove_all_files(dir: &PathBuf) -> () {
    info!("Removing all files in directory: {:?}", dir);
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            info!("Removing file: {:?}", path);
            fs::remove_file(path).unwrap();
        }
    }
}

/// Strips ANSI color codes from a string using a regex
/// This is useful for comparing strings with ANSI color codes to strings without
pub fn strip_ansi_codes(s: &str) -> String {
    let ansi_regex = Regex::new(r"\x1b\[([0-9]{1,2}(;[0-9]{0,2})*)m").unwrap();
    ansi_regex.replace_all(s, "").to_string()
}

/// Parses two strings into ASTs and compares them for equality
pub fn parse_and_compare_ast(first: &String, second: &String) -> Result<bool, syn::Error> {
    let first_ast: File = parse_file(&first)?;
    let second_ast: File = parse_file(&second)?;

    // Convert both ASTs back into token stres for comparison
    // FIXME this is sometimes buggy and is convinced that the two files are
    // different when they are infact the same
    let first_tokens: String = first_ast.into_token_stream().to_string();
    let second_tokens: String = second_ast.into_token_stream().to_string();

    Ok(first_tokens == second_tokens)
}

/// Prints the differences between two files to stdout
pub fn print_file_diff(expected_file_path: &str, output_file_path: &str) -> Result<(), std::io::Error> {
    let expected_content = fs::read_to_string(expected_file_path)?;
    let output_content = fs::read_to_string(output_file_path)?;

    if expected_content != output_content {
        println!("Differences found between expected and output:");
        for diff in diff::lines(&expected_content, &output_content) {
            match diff {
                diff::Result::Left(l) => println!("{}", format!("- {}", l).red()), // Expected but not in output
                diff::Result::Right(r) => println!("{}", format!("+ {}", r).green()), // In output but not in expected
                diff::Result::Both(b, _) => println!("{}", format!("  {}", b)), // Same in both
            }
        }
    } else {
        println!("{}", "No differences found.".green());
    }

    Ok(())
}