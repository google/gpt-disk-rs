// Copyright 2023> Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{env, process};
use uguid::Guid;

const USAGE: &str = r#"
usage: guid_info <guid>
the <guid> format is "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
where each `x` is a hex digit (any of `0-9`, `a-f`, or `A-F`).
"#;

fn format_bytes(bytes: &[u8]) -> String {
    let mut s = String::new();
    for (i, byte) in bytes.iter().enumerate() {
        if i != 0 && (i % 2) == 0 {
            s.push(' ');
        }
        s += &format!("{byte:02x}");
    }
    s
}

fn format_guid(guid: Guid) -> String {
    format!(
        "guid: {guid}
  time_low: {time_low}
  time_mid: {time_mid}
  time_high_and_version: {time_high_and_version}
  clock_seq_high_and_reserved: {clock_seq_high_and_reserved}
  clock_seq_low: {clock_seq_low}
  node: {node}",
        time_low = format_bytes(&guid.time_low()),
        time_mid = format_bytes(&guid.time_mid()),
        time_high_and_version = format_bytes(&guid.time_high_and_version()),
        clock_seq_high_and_reserved =
            format_bytes(&[guid.clock_seq_high_and_reserved()]),
        clock_seq_low = format_bytes(&[guid.clock_seq_low()]),
        node = format_bytes(&guid.node())
    )
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("{}", USAGE.trim());
        process::exit(1);
    }

    let arg = &args[1];
    match arg.parse::<Guid>() {
        Ok(guid) => {
            println!("{}", format_guid(guid));
        }
        Err(err) => {
            println!("invalid input: {err}");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(&[]), "");
        assert_eq!(format_bytes(&[0]), "00");
        assert_eq!(format_bytes(&[0x12, 0x34]), "1234");
        assert_eq!(format_bytes(&[0x12, 0x34, 0x56, 0x78]), "1234 5678");
    }
}
