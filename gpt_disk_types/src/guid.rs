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

/// Macro replacement for the `?` operator, which cannot be used in
/// const functions.
macro_rules! mtry {
    ($expr:expr $(,)?) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return Err(err);
            }
        }
    };
}

/// Parse a hexadecimal ASCII character as a `u8`.
const fn parse_byte_from_ascii_char(c: u8) -> Result<u8, GuidFromStrError> {
    match c {
        b'0' => Ok(0x0),
        b'1' => Ok(0x1),
        b'2' => Ok(0x2),
        b'3' => Ok(0x3),
        b'4' => Ok(0x4),
        b'5' => Ok(0x5),
        b'6' => Ok(0x6),
        b'7' => Ok(0x7),
        b'8' => Ok(0x8),
        b'9' => Ok(0x9),
        b'a' | b'A' => Ok(0xa),
        b'b' | b'B' => Ok(0xb),
        b'c' | b'C' => Ok(0xc),
        b'd' | b'D' => Ok(0xd),
        b'e' | b'E' => Ok(0xe),
        b'f' | b'F' => Ok(0xf),
        _ => Err(GuidFromStrError),
    }
}

/// Parse a pair of hexadecimal ASCII characters as a `u8`. For example,
/// `(b'1', b'a')` is parsed as `0x1a`.
const fn parse_byte_from_ascii_char_pair(
    a: u8,
    b: u8,
) -> Result<u8, GuidFromStrError> {
    let a = mtry!(parse_byte_from_ascii_char(a));
    let b = mtry!(parse_byte_from_ascii_char(b));
    Ok(a << 4 | b)
}

/// Parse a pair of hexadecimal ASCII characters at position `start` as
/// a `u8`.
const fn parse_byte_from_ascii_str_at(
    s: &[u8],
    start: usize,
) -> Result<u8, GuidFromStrError> {
    parse_byte_from_ascii_char_pair(s[start], s[start + 1])
}

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

    /// Parse a GUID from a string.
    ///
    /// This is functionally the same as [`Guid::from_str`], but is
    /// exposed separately to provide a `const` method for parsing.
    pub const fn try_parse(s: &str) -> Result<Self, GuidFromStrError> {
        // Treat input as ASCII.
        let s = s.as_bytes();

        if s.len() != 36 {
            return Err(GuidFromStrError);
        }

        let sep = b'-';
        if s[8] != sep || s[13] != sep || s[18] != sep || s[23] != sep {
            return Err(GuidFromStrError);
        }

        Ok(Guid {
            time_low: U32Le([
                mtry!(parse_byte_from_ascii_str_at(s, 6)),
                mtry!(parse_byte_from_ascii_str_at(s, 4)),
                mtry!(parse_byte_from_ascii_str_at(s, 2)),
                mtry!(parse_byte_from_ascii_str_at(s, 0)),
            ]),
            time_mid: U16Le([
                mtry!(parse_byte_from_ascii_str_at(s, 11)),
                mtry!(parse_byte_from_ascii_str_at(s, 9)),
            ]),
            time_high_and_version: U16Le([
                mtry!(parse_byte_from_ascii_str_at(s, 16)),
                mtry!(parse_byte_from_ascii_str_at(s, 14)),
            ]),
            clock_seq_high_and_reserved: mtry!(parse_byte_from_ascii_str_at(
                s, 19
            )),
            clock_seq_low: mtry!(parse_byte_from_ascii_str_at(s, 21)),
            node: [
                mtry!(parse_byte_from_ascii_str_at(s, 24)),
                mtry!(parse_byte_from_ascii_str_at(s, 26)),
                mtry!(parse_byte_from_ascii_str_at(s, 28)),
                mtry!(parse_byte_from_ascii_str_at(s, 30)),
                mtry!(parse_byte_from_ascii_str_at(s, 32)),
                mtry!(parse_byte_from_ascii_str_at(s, 34)),
            ],
        })
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

/// Error type for [`Guid::try_parse`] and [`Guid::from_str`].
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
        Self::try_parse(s)
    }
}

/// Create a [`Guid`] from a string at compile time.
///
/// # Examples
///
/// ```
/// use gpt_disk_types::{guid, Guid, U16Le, U32Le};
/// assert_eq!(
///     guid!("01234567-89ab-cdef-0123-456789abcdef"),
///     Guid::new(
///         U32Le::from_u32(0x01234567),
///         U16Le::from_u16(0x89ab),
///         U16Le::from_u16(0xcdef),
///         0x01,
///         0x23,
///         [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
///     )
/// );
/// ```
#[macro_export]
macro_rules! guid {
    ($s:literal) => {
        match $crate::Guid::try_parse($s) {
            Ok(g) => g,
            Err(_) => panic!("invalid GUID string"),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(parse_byte_from_ascii_char_pair(b'1', b'a'), Ok(0x1a));
        assert_eq!(parse_byte_from_ascii_char_pair(b'8', b'f'), Ok(0x8f));
    }
}
