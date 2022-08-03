// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use common::check_derives;
use gpt_disk_types::{guid, Guid, GuidFromStrError};

#[test]
fn test_guid() {
    check_derives::<Guid>();

    let guid = Guid::new(
        0x01234567_u32.to_le_bytes(),
        0x89ab_u16.to_le_bytes(),
        0xcdef_u16.to_le_bytes(),
        0x01,
        0x23,
        [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    );

    // To byte array.
    assert_eq!(
        guid.to_bytes(),
        [
            0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45,
            0x67, 0x89, 0xab, 0xcd, 0xef
        ]
    );

    // Formatting.
    assert_eq!(
        guid.to_ascii_hex_lower(),
        *b"01234567-89ab-cdef-0123-456789abcdef"
    );
    assert_eq!(guid.to_string(), "01234567-89ab-cdef-0123-456789abcdef");

    // Parsing.
    assert_eq!(
        "01234567-89ab-cdef-0123-456789abcdef"
            .parse::<Guid>()
            .unwrap(),
        guid
    );
    assert_eq!(
        Guid::try_parse("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        guid
    );

    // Macro.
    assert_eq!(guid!("01234567-89ab-cdef-0123-456789abcdef"), guid);
}

#[test]
fn test_guid_error() {
    check_derives::<GuidFromStrError>();
}

/// Inner module that only imports the `guid!` macro.
mod inner {
    use gpt_disk_types::guid;

    /// Test that the `guid!` macro works without importing anything
    /// else.
    #[test]
    fn test_guid_macro_paths() {
        guid!("01234567-89ab-cdef-0123-456789abcdef");
    }
}
