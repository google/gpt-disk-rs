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

#[cfg(feature = "serde")]
use serde_test::Token;
#[cfg(feature = "serde")]
use uguid::{guid, Guid};

#[cfg(feature = "serde")]
#[test]
fn test_serde() {
    let guid = guid!("01234567-89ab-cdef-0123-456789abcdef");

    serde_test::assert_tokens(
        &guid,
        &[Token::Str("01234567-89ab-cdef-0123-456789abcdef")],
    );

    serde_test::assert_de_tokens_error::<Guid>(
        &[Token::Str("1234")],
        "GUID hex string does not match expected format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"");

    serde_test::assert_de_tokens_error::<Guid>(
        &[Token::U64(1234)],
        "invalid type: integer `1234`, expected a string in the format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"");
}
