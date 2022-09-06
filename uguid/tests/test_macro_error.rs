// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tests for macro errors from the guid macros. Ordinarily this would
//! be done with trybuild, but there are a few problems:
//!
//! 1. Due to <https://github.com/dtolnay/trybuild/issues/171>, we get
//!    errors from optional dependencies.
//!
//! 2. The error output includes the path of `panic.rs` in the core
//!    library, and that path is likely different on each installation.
//!
//! 3. The error output includes the line number of the uguid source
//!    file containing `parse_or_panic`, which makes the test need frequent
//!    updating.
//!
//! Trying to fix trybuild for our particular situation seems somewhat
//! unlikely to pass muster with the maintainer, so for now just
//! hand roll this test.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn compile_and_capture(src_path: &str) -> String {
    let tmp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let uguid_dir = env!("CARGO_MANIFEST_DIR");
    let crate_dir = tmp_dir.join("uguid_compilation_test");
    let src_dir = crate_dir.join("src");
    let cargo_toml_path = crate_dir.join("Cargo.toml");
    let _ = fs::remove_dir_all(&crate_dir);
    fs::create_dir(&crate_dir).unwrap();
    fs::create_dir(&src_dir).unwrap();
    fs::write(
        &cargo_toml_path,
        r#"
        [package]
        name = "uguid_compilation_test"
        version = "0.0.0"
        edition = "2021"

        [dependencies]
        uguid = { path = "UGUID_DIR" }

        [workspace]
        # Empty section to prevent conflict with parent workspace.
    "#
        .replace("UGUID_DIR", uguid_dir),
    )
    .unwrap();
    fs::copy(src_path, src_dir.join("lib.rs")).unwrap();
    let output = Command::new("cargo")
        .args(&[
            // Disable color so that the output doesn't can't get
            // polluted with terminal control codes.
            "--color=never",
            "check",
            "--quiet",
            "--manifest-path",
        ])
        .arg(cargo_toml_path)
        .output()
        .unwrap();
    // The build is supposed to fail.
    assert!(!output.status.success());

    String::from_utf8(output.stderr).unwrap()
}

fn modify_actual(orig_actual: &str, expected: &str) -> String {
    let actual: Vec<_> = orig_actual.lines().collect();
    let expected: Vec<_> = expected.lines().collect();
    if actual.len() != expected.len() {
        return orig_actual.to_owned();
    }

    let mut modified_actual = Vec::new();

    for (actual, expected) in actual.iter().zip(expected.iter()) {
        let actual = actual.to_string();

        // If the expected line contains "IGNORE_REST", chop off
        // everything past that in the actual line and add
        // "IGNORE_REST", making it match the expected line exactly.
        let ignore_rest = "IGNORE_REST";
        if let Some(p) = expected.find(ignore_rest) {
            if actual.len() <= p {
                // Actual line is too short.
                modified_actual.push(actual);
            } else {
                modified_actual.push(format!(
                    "{}{}",
                    &actual[..p],
                    ignore_rest
                ));
            }
        } else {
            modified_actual.push(actual);
        }
    }

    modified_actual.push(String::new());
    modified_actual.join("\n")
}

fn print_mismatch(actual: &str, expected: &str, modified_actual: &str) {
    let sep = "//////////////////////////////////////////////////";
    println!("EXPECTED:");
    print!("{expected}");
    println!("{sep}");
    println!("ACTUAL:");
    print!("{actual}");
    println!("{sep}");
    println!("MODIFIED ACTUAL:");
    print!("{modified_actual}");
    println!("{sep}");
}

#[test]
fn test_compilation_errors() {
    // TODO: generalize this and test some more error cases.
    let actual = compile_and_capture("tests/ui/guid_len.rs");
    let expected = fs::read_to_string("tests/ui/guid_len.stderr").unwrap();
    let modified_actual = modify_actual(&actual, &expected);
    if modified_actual != expected {
        print_mismatch(&actual, &expected, &modified_actual);
        panic!("mismatch in {}", "todo");
    }
}
