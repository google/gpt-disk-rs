// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
