// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(feature = "serde")]

use serde_test::Token;
use uguid::{aligned_guid, guid, AlignedGuid, Guid};

#[test]
fn test_serde_unaligned() {
    let guid = guid!("01234567-89ab-cdef-0123-456789abcdef");

    serde_test::assert_tokens(
        &guid,
        &[Token::Str("01234567-89ab-cdef-0123-456789abcdef")],
    );

    serde_test::assert_de_tokens_error::<Guid>(
        &[Token::Str("1234")],
        "GUID string has wrong length (expected 36 bytes)",
    );

    serde_test::assert_de_tokens_error::<Guid>(
        &[Token::U64(1234)],
        "invalid type: integer `1234`, expected a string in the format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"");
}

#[test]
fn test_serde_aligned() {
    let guid = aligned_guid!("01234567-89ab-cdef-0123-456789abcdef");

    serde_test::assert_tokens(
        &guid,
        &[Token::Str("01234567-89ab-cdef-0123-456789abcdef")],
    );

    serde_test::assert_de_tokens_error::<Guid>(
        &[Token::Str("1234")],
        "GUID string has wrong length (expected 36 bytes)",
    );

    serde_test::assert_de_tokens_error::<Guid>(
        &[Token::U64(1234)],
        "invalid type: integer `1234`, expected a string in the format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"");
}
