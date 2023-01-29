// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::fmt::{self, Display, Formatter};

/// Error type for [`Guid::try_parse`] and [`Guid::from_str`].
///
/// If the `std` feature is enabled, this type implements the [`Error`]
/// trait.
///
/// [`Error`]: std::error::Error
/// [`Guid::from_str`]: core::str::FromStr::from_str
/// [`Guid::try_parse`]: crate::Guid::try_parse
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GuidFromStrError {
    /// Input has the wrong length, expected 36 bytes.
    Length,

    /// Input is missing a separator (`-`) at this byte index.
    Separator(u8),

    /// Input contains invalid ASCII hex at this byte index.
    Hex(u8),
}

impl Default for GuidFromStrError {
    fn default() -> Self {
        Self::Length
    }
}

impl Display for GuidFromStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length => {
                f.write_str("GUID string has wrong length (expected 36 bytes)")
            }
            Self::Separator(index) => write!(
                f,
                "GUID string is missing a separator (`-`) at index {index}",
            ),
            Self::Hex(index) => {
                write!(
                    f,
                    "GUID string contains invalid ASCII hex at index {index}",
                )
            }
        }
    }
}
