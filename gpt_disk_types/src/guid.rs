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

use crate::{U16Le, U32Le};
use bytemuck::{Pod, Zeroable};
use core::fmt::{self, Display, Formatter};
use core::num::ParseIntError;
use core::str::FromStr;

/// Globally-unique identifier.
///
/// The format is described in Appendix A of the UEFI Specification.
///
/// # Examples
///
/// ```
/// use gpt_disk_types::{Guid, U16Le, U32Le};
/// let guid = Guid::new(
///     U32Le::from_u32(0x01234567),
///     U16Le::from_u16(0x89ab),
///     U16Le::from_u16(0xcdef),
///     0x01,
///     0x23,
///     [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
/// );
/// assert_eq!(guid.to_string(), "01234567-89ab-cdef-0123-456789abcdef");
///
/// assert_eq!(
///     "01234567-89ab-cdef-0123-456789abcdef"
///         .parse::<Guid>()
///         .unwrap(),
///     guid
/// );
/// ```
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable,
)]
#[repr(C)]
#[allow(missing_docs)]
pub struct Guid {
    pub time_low: U32Le,
    pub time_mid: U16Le,
    pub time_high_and_version: U16Le,
    pub clock_seq_high_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

impl Guid {
    /// GUID with all fields set to zero.
    pub const ZERO: Self = Self::new(
        U32Le::from_u32(0),
        U16Le::from_u16(0),
        U16Le::from_u16(0),
        0,
        0,
        [0, 0, 0, 0, 0, 0],
    );

    /// Create a new GUID.
    #[must_use]
    pub const fn new(
        time_low: U32Le,
        time_mid: U16Le,
        time_high_and_version: U16Le,
        clock_seq_high_and_reserved: u8,
        clock_seq_low: u8,
        node: [u8; 6],
    ) -> Self {
        Self {
            time_low,
            time_mid,
            time_high_and_version,
            clock_seq_high_and_reserved,
            clock_seq_low,
            node,
        }
    }
}

impl Default for Guid {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f, "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.time_low.0[3],
            self.time_low.0[2],
            self.time_low.0[1],
            self.time_low.0[0],

            self.time_mid.0[1],
            self.time_mid.0[0],

            self.time_high_and_version.0[1],
            self.time_high_and_version.0[0],

            self.clock_seq_high_and_reserved,
            self.clock_seq_low,

            self.node[0],
            self.node[1],
            self.node[2],
            self.node[3],
            self.node[4],
            self.node[5],
        )
    }
}

/// Error type for [`Guid::from_str`].
///
/// If the `std` feature is enabled, this type implements the [`Error`]
/// trait.
///
/// [`Error`]: std::error::Error
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GuidFromStrError;

impl Display for GuidFromStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("GUID hex string does not match expected format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"")
    }
}

impl From<ParseIntError> for GuidFromStrError {
    fn from(_err: ParseIntError) -> Self {
        Self
    }
}

impl FromStr for Guid {
    type Err = GuidFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 36 {
            return Err(GuidFromStrError);
        }

        let sep = b'-';
        let b = s.as_bytes();
        if b[8] != sep || b[13] != sep || b[18] != sep || b[23] != sep {
            return Err(GuidFromStrError);
        }

        let parse_byte = |start| u8::from_str_radix(&s[start..start + 2], 16);

        Ok(Guid {
            time_low: U32Le([
                parse_byte(6)?,
                parse_byte(4)?,
                parse_byte(2)?,
                parse_byte(0)?,
            ]),
            time_mid: U16Le([parse_byte(11)?, parse_byte(9)?]),
            time_high_and_version: U16Le([parse_byte(16)?, parse_byte(14)?]),
            clock_seq_high_and_reserved: parse_byte(19)?,
            clock_seq_low: parse_byte(21)?,
            node: [
                parse_byte(24)?,
                parse_byte(26)?,
                parse_byte(28)?,
                parse_byte(30)?,
                parse_byte(32)?,
                parse_byte(34)?,
            ],
        })
    }
}
