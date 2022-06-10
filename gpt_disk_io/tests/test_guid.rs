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

mod common;

use common::check_derives;
use gpt_disk_types::{guid, Guid, GuidFromStrError, U16Le, U32Le};

#[test]
fn test_guid() {
    check_derives::<Guid>();

    let guid = Guid::new(
        U32Le::from_u32(0x01234567),
        U16Le::from_u16(0x89ab),
        U16Le::from_u16(0xcdef),
        0x01,
        0x23,
        [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    );
    // Formatting.
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
    use gpt_disk_types::guid;

    /// Test that the `guid!` macro works without importing anything
    /// else.
    #[test]
    fn test_guid_macro_paths() {
        guid!("01234567-89ab-cdef-0123-456789abcdef");
    }
}
