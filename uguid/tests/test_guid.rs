// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use uguid::{guid, Guid, GuidFromStrError};

#[test]
fn test_guid() {
    // Constructors.
    let guid = Guid::new(
        0x01234567_u32.to_le_bytes(),
        0x89ab_u16.to_le_bytes(),
        0xcdef_u16.to_le_bytes(),
        0x01,
        0x23,
        [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    );
    let guid2 = Guid::from_bytes([
        0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67,
        0x89, 0xab, 0xcd, 0xef,
    ]);
    assert_eq!(guid, guid2);

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
    // Wrong length.
    let s = "01234567-89ab-cdef-0123-456789abcdef0";
    assert_eq!(s.len(), 37);
    assert_eq!(s.parse::<Guid>(), Err(GuidFromStrError));

    // Wrong separator.
    let s = "01234567x89ab-cdef-0123-456789abcdef";
    assert_eq!(s.parse::<Guid>(), Err(GuidFromStrError));

    // Invalid hex.
    let s = "g1234567-89ab-cdef-0123-456789abcdef";
    assert_eq!(s.parse::<Guid>(), Err(GuidFromStrError));
}

/// Inner module that only imports the `guid!` macro.
mod inner {
    use uguid::guid;

    /// Test that the `guid!` macro works without importing anything
    /// else.
    #[test]
    fn test_guid_macro_paths() {
        guid!("01234567-89ab-cdef-0123-456789abcdef");
    }
}
