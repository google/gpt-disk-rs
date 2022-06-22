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

//! Library providing a GUID (Globally Unique Identifier) type. The
//! format is described in Appendix A of the UEFI
//! Specification. This format of GUID is also used in Microsoft
//! Windows.
//!
//! # Features
//!
//! No features are enabled by default.
//!
//! * `serde`: Implements serde's `Serialize` and `Deserialize` traits for `Guid`.
//! * `std`: Provides `std::error::Error` implementation for the error type.
//!
//! # Examples
//!
//! Construct a GUID at compile time with the `guid!` macro:
//!
//! ```
//! use uguid::guid;
//!
//! let guid = guid!("01234567-89ab-cdef-0123-456789abcdef");
//! ```
//!
//! Parse a GUID at runtime from a string:
//!
//! ```
//! use uguid::Guid;
//!
//! let guid: Guid = "01234567-89ab-cdef-0123-456789abcdef".parse().unwrap();
//! ```
//!
//! Construct a GUID from its components or a byte array:
//!
//! ```
//! use uguid::Guid;
//!
//! let guid1 = Guid::new(
//!     0x01234567_u32.to_le_bytes(),
//!     0x89ab_u16.to_le_bytes(),
//!     0xcdef_u16.to_le_bytes(),
//!     0x01,
//!     0x23,
//!     [0x45, 0x67, 0x89, 0xab, 0xcd, 0xef],
//! );
//! let guid2 = Guid::from_bytes([
//!     0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45, 0x67,
//!     0x89, 0xab, 0xcd, 0xef,
//! ]);
//! assert_eq!(guid1, guid2);
//! ```
//!
//! Convert to a string or a byte array:
//!
//! ```
//! use uguid::guid;
//!
//! let guid = guid!("01234567-89ab-cdef-0123-456789abcdef");
//! assert_eq!(guid.to_string(), "01234567-89ab-cdef-0123-456789abcdef");
//! assert_eq!(
//!     guid.to_bytes(),
//!     [
//!         0x67, 0x45, 0x23, 0x01, 0xab, 0x89, 0xef, 0xcd, 0x01, 0x23, 0x45,
//!         0x67, 0x89, 0xab, 0xcd, 0xef
//!     ]
//! );
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::as_conversions)]
#![allow(clippy::missing_errors_doc)]

#[cfg(feature = "serde")]
mod guid_serde;

use bytemuck::{Pod, Zeroable};
use core::fmt::{self, Display, Formatter};
use core::str::{self, FromStr};

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

const fn byte_to_ascii_hex_lower(byte: u8) -> (u8, u8) {
    let mut l = byte & 0xf;
    let mut h = byte >> 4;
    if l <= 9 {
        l += b'0';
    } else {
        l += b'a' - 10;
    }
    if h <= 9 {
        h += b'0';
    } else {
        h += b'a' - 10;
    }
    (h, l)
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
/// The format is described in Appendix A of the UEFI
/// Specification. Note that the first three fields are little-endian.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Pod, Zeroable,
)]
#[repr(C)]
#[allow(missing_docs)]
pub struct Guid {
    pub time_low: [u8; 4],
    pub time_mid: [u8; 2],
    pub time_high_and_version: [u8; 2],
    pub clock_seq_high_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

impl Guid {
    /// GUID with all fields set to zero.
    pub const ZERO: Self =
        Self::new([0, 0, 0, 0], [0, 0], [0, 0], 0, 0, [0, 0, 0, 0, 0, 0]);

    /// Create a new GUID.
    #[must_use]
    pub const fn new(
        time_low: [u8; 4],
        time_mid: [u8; 2],
        time_high_and_version: [u8; 2],
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
            time_low: [
                mtry!(parse_byte_from_ascii_str_at(s, 6)),
                mtry!(parse_byte_from_ascii_str_at(s, 4)),
                mtry!(parse_byte_from_ascii_str_at(s, 2)),
                mtry!(parse_byte_from_ascii_str_at(s, 0)),
            ],
            time_mid: [
                mtry!(parse_byte_from_ascii_str_at(s, 11)),
                mtry!(parse_byte_from_ascii_str_at(s, 9)),
            ],
            time_high_and_version: [
                mtry!(parse_byte_from_ascii_str_at(s, 16)),
                mtry!(parse_byte_from_ascii_str_at(s, 14)),
            ],
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

    /// Create a GUID from a 16-byte array. No changes to byte order are made.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            time_low: [bytes[0], bytes[1], bytes[2], bytes[3]],
            time_mid: [bytes[4], bytes[5]],
            time_high_and_version: [bytes[6], bytes[7]],
            clock_seq_high_and_reserved: bytes[8],
            clock_seq_low: bytes[9],
            node: [
                bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ],
        }
    }

    /// Convert to a 16-byte array.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 16] {
        [
            self.time_low[0],
            self.time_low[1],
            self.time_low[2],
            self.time_low[3],
            self.time_mid[0],
            self.time_mid[1],
            self.time_high_and_version[0],
            self.time_high_and_version[1],
            self.clock_seq_high_and_reserved,
            self.clock_seq_low,
            self.node[0],
            self.node[1],
            self.node[2],
            self.node[3],
            self.node[4],
            self.node[5],
        ]
    }

    /// Convert to a lower-case hex ASCII string.
    ///
    /// The output is in "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" format.
    #[must_use]
    pub const fn to_ascii_hex_lower(self) -> [u8; 36] {
        let mut buf = [0; 36];
        (buf[0], buf[1]) = byte_to_ascii_hex_lower(self.time_low[3]);
        (buf[2], buf[3]) = byte_to_ascii_hex_lower(self.time_low[2]);
        (buf[4], buf[5]) = byte_to_ascii_hex_lower(self.time_low[1]);
        (buf[6], buf[7]) = byte_to_ascii_hex_lower(self.time_low[0]);
        buf[8] = b'-';
        (buf[9], buf[10]) = byte_to_ascii_hex_lower(self.time_mid[1]);
        (buf[11], buf[12]) = byte_to_ascii_hex_lower(self.time_mid[0]);
        buf[13] = b'-';
        (buf[14], buf[15]) =
            byte_to_ascii_hex_lower(self.time_high_and_version[1]);
        (buf[16], buf[17]) =
            byte_to_ascii_hex_lower(self.time_high_and_version[0]);
        buf[18] = b'-';
        (buf[19], buf[20]) =
            byte_to_ascii_hex_lower(self.clock_seq_high_and_reserved);
        (buf[21], buf[22]) = byte_to_ascii_hex_lower(self.clock_seq_low);
        buf[23] = b'-';
        (buf[24], buf[25]) = byte_to_ascii_hex_lower(self.node[0]);
        (buf[26], buf[27]) = byte_to_ascii_hex_lower(self.node[1]);
        (buf[28], buf[29]) = byte_to_ascii_hex_lower(self.node[2]);
        (buf[30], buf[31]) = byte_to_ascii_hex_lower(self.node[3]);
        (buf[32], buf[33]) = byte_to_ascii_hex_lower(self.node[4]);
        (buf[34], buf[35]) = byte_to_ascii_hex_lower(self.node[5]);
        buf
    }
}

impl Default for Guid {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Display for Guid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ascii = self.to_ascii_hex_lower();
        // OK to unwrap since the ascii output is valid utf-8.
        let s = str::from_utf8(&ascii).unwrap();
        f.write_str(s)
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

impl FromStr for Guid {
    type Err = GuidFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for GuidFromStrError {}

/// Create a [`Guid`] from a string at compile time.
///
/// # Examples
///
/// ```
/// use uguid::{guid, Guid};
/// assert_eq!(
///     guid!("01234567-89ab-cdef-0123-456789abcdef"),
///     Guid::new(
///         0x01234567_u32.to_le_bytes(),
///         0x89ab_u16.to_le_bytes(),
///         0xcdef_u16.to_le_bytes(),
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

    #[test]
    fn test_to_ascii() {
        assert_eq!(byte_to_ascii_hex_lower(0x1f), (b'1', b'f'));
        assert_eq!(byte_to_ascii_hex_lower(0xf1), (b'f', b'1'));
    }
}
