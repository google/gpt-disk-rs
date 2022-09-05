// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::mem;
use uguid::{AlignedGuid, Guid};

/// Test that both GUID structs have the correct alignment.
#[test]
fn test_alignment() {
    assert_eq!(mem::align_of::<Guid>(), 1);
    assert_eq!(mem::align_of::<AlignedGuid>(), 8);
}
