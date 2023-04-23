// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::process::{exit, Command};

const FEAT_OPTIONS: [bool; 2] = [false, true];
const FEAT_BYTEMUCK: &str = "bytemuck";
const FEAT_SERDE: &str = "serde";
const FEAT_STD: &str = "std";

fn run_cmd(mut cmd: Command) {
    println!("Running: {}", format!("{cmd:?}").replace('"', ""));
    let status = cmd.status().expect("failed to launch");
    if !status.success() {
        panic!("command failed: {status}");
    }
}

#[derive(Clone, Copy)]
enum CargoAction {
    Test,
    Lint,
}

impl CargoAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lint => "clippy",
            Self::Test => "test",
        }
    }
}

fn get_cargo_cmd(
    action: CargoAction,
    package: &str,
    features: &[&str],
) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args([action.as_str(), "--package", package]);
    if !features.is_empty() {
        cmd.args(["--features", &features.join(",")]);
    }
    match action {
        CargoAction::Test => {}
        CargoAction::Lint => {
            cmd.args(["--", "-D", "warnings"]);
        }
    }
    cmd
}

fn test_package(package: &str, features: &[&str]) {
    run_cmd(get_cargo_cmd(CargoAction::Lint, package, features));
    run_cmd(get_cargo_cmd(CargoAction::Test, package, features));
}

fn test_uguid() {
    for feat_bytemuck in FEAT_OPTIONS {
        for feat_serde in FEAT_OPTIONS {
            for feat_std in FEAT_OPTIONS {
                let mut features = Vec::new();
                if feat_bytemuck {
                    features.push(FEAT_BYTEMUCK);
                }
                if feat_serde {
                    features.push(FEAT_SERDE);
                }
                if feat_std {
                    features.push(FEAT_STD);
                }

                test_package("uguid", &features);
            }
        }
    }
}

fn test_gpt_disk_types() {
    for feat_bytemuck in FEAT_OPTIONS {
        for feat_std in FEAT_OPTIONS {
            let mut features = Vec::new();
            if feat_bytemuck {
                features.push(FEAT_BYTEMUCK);
            }
            if feat_std {
                features.push(FEAT_STD);
            }

            test_package("gpt_disk_types", &features);
        }
    }
}

fn test_gpt_disk_io() {
    for feat_std in FEAT_OPTIONS {
        let mut features = Vec::new();
        if feat_std {
            features.push("std");
        }

        test_package("gpt_disk_io", &features);
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let arg_test_all = "test_all";
    let arg_test_uguid = "test_uguid";
    let arg_test_gpt_disk_types = "test_gpt_disk_types";
    let arg_test_gpt_disk_io = "test_gpt_disk_io";
    let actions = &[
        arg_test_all,
        arg_test_uguid,
        arg_test_gpt_disk_types,
        arg_test_gpt_disk_io,
    ];
    if args.len() != 2 || !actions.contains(&args[1].as_ref()) {
        println!("usage: cargo xtask [{}]", actions.join("|"));
        exit(1);
    }

    let action = &args[1];
    if action == arg_test_all || action == arg_test_uguid {
        test_uguid();
    }
    if action == arg_test_all || action == arg_test_gpt_disk_types {
        test_gpt_disk_types();
    }
    if action == arg_test_all || action == arg_test_gpt_disk_io {
        test_gpt_disk_io();
    }
}
