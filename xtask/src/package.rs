// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Public packages in the workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Package {
    Uguid,
    GptDiskTypes,
    GptDiskIo,
}

impl Package {
    /// Package name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Uguid => "uguid",
            Self::GptDiskTypes => "gpt_disk_types",
            Self::GptDiskIo => "gpt_disk_io",
        }
    }
}
