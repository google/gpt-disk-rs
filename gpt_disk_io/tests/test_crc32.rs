// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod common;

use common::check_derives;
use gpt_disk_types::{Crc32, U32Le};

#[test]
fn test_crc32_display() {
    check_derives::<Crc32>();

    let crc = Crc32(U32Le([0x12, 0x34, 0x56, 0x78]));
    assert_eq!(format!("{crc:#x}"), "0x78563412");
    assert_eq!(format!("{crc}"), "0x78563412");
}
