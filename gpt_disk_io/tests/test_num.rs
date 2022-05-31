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

use gpt_disk_types::{U16Le, U32Le, U64Le};

#[test]
fn test_u16le() {
    let mut v = U16Le::from_u16(123);
    assert_eq!(v.to_u16(), 123);
    v.set(0xabc);
    assert_eq!(v.to_u16(), 0xabc);
    assert_eq!(format!("{v:x?}"), "abc");
    assert_eq!(format!("{v}"), "2748");
}

#[test]
fn test_u32le() {
    let mut v = U32Le::from_u32(123);
    assert_eq!(v.to_u32(), 123);
    v.set(0xabc);
    assert_eq!(v.to_u32(), 0xabc);
    assert_eq!(format!("{v:x?}"), "abc");
    assert_eq!(format!("{v}"), "2748");
}

#[test]
fn test_u64le() {
    let mut v = U64Le::from_u64(123);
    assert_eq!(v.to_u64(), 123);
    v.set(0xabc);
    assert_eq!(v.to_u64(), 0xabc);
    assert_eq!(format!("{v:x?}"), "abc");
    assert_eq!(format!("{v}"), "2748");
}

#[test]
fn test_num_display() {
    let n = U16Le::from_u16(0x1234);
    assert_eq!(format!("{n} {n:x} {n:#x}"), "4660 1234 0x1234");

    let n = U32Le::from_u32(0x1234_5678);
    assert_eq!(format!("{n} {n:x} {n:#x}"), "305419896 12345678 0x12345678");

    let n = U64Le::from_u64(0x1234_5678_9abc_def0);
    assert_eq!(
        format!("{n} {n:x} {n:#x}"),
        "1311768467463790320 123456789abcdef0 0x123456789abcdef0"
    );
}
