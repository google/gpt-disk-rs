// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use uguid::{aligned_guid, AlignedGuid, GuidFromStrError};

#[test]
fn test_guid() {
    // Constructors.
    let guid = AlignedGuid {
        time_low: 0x01234567_u32.to_le_bytes(),
        time_mid: 0x89ab_u16.to_le_bytes(),
        time_high_and_version: 0xcdef_u16.to_le_bytes(),
        clock_seq_high_and_reserved: 0x01,
        clock_seq_low: 0x23,
        node: [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    };
    let guid2 = AlignedGuid::new(
        0x01234567_u32.to_le_bytes(),
        0x89ab_u16.to_le_bytes(),
        0xcdef_u16.to_le_bytes(),
        0x01,
        0x23,
        [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    );
    let guid3 = AlignedGuid::from_bytes([
        0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67,
        0x89, 0xab, 0xcd, 0xef,
    ]);
    assert_eq!(guid, guid2);
    assert_eq!(guid, guid3);

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
            .parse::<AlignedGuid>()
            .unwrap(),
        guid
    );
    assert_eq!(
        AlignedGuid::try_parse("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        guid
    );

    // Macro.
    assert_eq!(aligned_guid!("01234567-89ab-cdef-0123-456789abcdef"), guid);
}

#[test]
fn test_guid_error() {
    // Wrong length.
    let s = "01234567-89ab-cdef-0123-456789abcdef0";
    assert_eq!(s.len(), 37);
    assert_eq!(s.parse::<AlignedGuid>(), Err(GuidFromStrError::Length));

    // Wrong separator.
    let s = "01234567089ab-cdef-0123-456789abcdef";
    assert_eq!(
        s.parse::<AlignedGuid>(),
        Err(GuidFromStrError::Separator(8))
    );
    let s = "01234567-89ab0cdef-0123-456789abcdef";
    assert_eq!(
        s.parse::<AlignedGuid>(),
        Err(GuidFromStrError::Separator(13))
    );
    let s = "01234567-89ab-cdef00123-456789abcdef";
    assert_eq!(
        s.parse::<AlignedGuid>(),
        Err(GuidFromStrError::Separator(18))
    );
    let s = "01234567-89ab-cdef-01230456789abcdef";
    assert_eq!(
        s.parse::<AlignedGuid>(),
        Err(GuidFromStrError::Separator(23))
    );

    // Invalid hex.
    let s = "g1234567-89ab-cdef-0123-456789abcdef";
    assert_eq!(s.parse::<AlignedGuid>(), Err(GuidFromStrError::Hex(0)));

    assert_eq!(
        GuidFromStrError::Length.to_string(),
        "GUID string has wrong length (expected 36 bytes)"
    );
    assert_eq!(
        GuidFromStrError::Separator(8).to_string(),
        "GUID string is missing a separator (`-`) at index 8"
    );
    assert_eq!(
        GuidFromStrError::Hex(10).to_string(),
        "GUID string contains invalid ASCII hex at index 10"
    );
}

/// Inner module that only imports the `guid!` macro.
mod inner {
    use uguid::aligned_guid;

    /// Test that the `guid!` macro works without importing anything
    /// else.
    #[test]
    fn test_guid_macro_paths() {
        let _g = aligned_guid!("01234567-89ab-cdef-0123-456789abcdef");
    }
}
