// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    GptPartitionEntryArrayError, GptPartitionEntrySizeError,
    GptPartitionNameFromStrError, GptPartitionNameSetCharError,
};
use std::error::Error;

impl Error for GptPartitionNameFromStrError {}

impl Error for GptPartitionNameSetCharError {}

impl Error for GptPartitionEntrySizeError {}

impl Error for GptPartitionEntryArrayError {}
