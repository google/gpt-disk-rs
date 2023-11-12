// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::{bail, Result};
use std::process::Command;

fn print_cmd(cmd: &Command) {
    println!("Running: {}", format!("{cmd:?}").replace('"', ""));
}

pub fn run_cmd(mut cmd: Command) -> Result<()> {
    print_cmd(&cmd);
    let status = cmd.status().expect("failed to launch");
    if status.success() {
        Ok(())
    } else {
        bail!("command failed: {status}");
    }
}

pub fn get_cmd_stdout(mut cmd: Command) -> Result<Vec<u8>> {
    print_cmd(&cmd);
    let output = cmd.output().expect("failed to launch");
    if output.status.success() {
        Ok(output.stdout)
    } else {
        bail!("command failed: {}", output.status);
    }
}
