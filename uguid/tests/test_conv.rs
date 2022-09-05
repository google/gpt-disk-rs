// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use uguid::{AlignedGuid, Guid};

/// Test the `From` conversions between the two GUID types.
#[test]
fn test_conversion() {
    let guid = Guid {
        time_low: 0x01234567_u32.to_le_bytes(),
        time_mid: 0x89ab_u16.to_le_bytes(),
        time_high_and_version: 0xcdef_u16.to_le_bytes(),
        clock_seq_high_and_reserved: 0x01,
        clock_seq_low: 0x23,
        node: [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    };

    let aligned_guid = AlignedGuid {
        time_low: 0x01234567_u32.to_le_bytes(),
        time_mid: 0x89ab_u16.to_le_bytes(),
        time_high_and_version: 0xcdef_u16.to_le_bytes(),
        clock_seq_high_and_reserved: 0x01,
        clock_seq_low: 0x23,
        node: [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
    };

    assert_eq!(AlignedGuid::from(guid), aligned_guid);
    assert_eq!(Guid::from(aligned_guid), guid);
}
