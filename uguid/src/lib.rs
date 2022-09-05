// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Library providing a GUID (Globally Unique Identifier) type. The
//! format is described in Appendix A of the UEFI
//! Specification. This format of GUID is also used in Microsoft
//! Windows.
//!
//! # Features
//!
//! No features are enabled by default.
//!
//! * `bytemuck`: Implements bytemuck's `Pod` and `Zeroable` traits for `Guid`.
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
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
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

mod util;

mod guid_parse;
pub use guid_parse::GuidFromStrError;

#[cfg(feature = "serde")]
mod guid_serde;

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

use core::fmt::{self, Display, Formatter};
use core::str::{self, FromStr};
use util::byte_to_ascii_hex_lower;

/// Globally-unique identifier.
///
/// The format is described in Appendix A of the UEFI
/// Specification. Note that the first three fields are little-endian.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C)]
pub struct Guid {
    /// The little-endian low field of the timestamp.
    pub time_low: [u8; 4],

    /// The little-endian middle field of the timestamp.
    pub time_mid: [u8; 2],

    /// The little-endian high field of the timestamp multiplexed with
    /// the version number.
    pub time_high_and_version: [u8; 2],

    /// The high field of the clock sequence multiplexed with the
    /// variant.
    pub clock_seq_high_and_reserved: u8,

    /// The low field of the clock sequence.
    pub clock_seq_low: u8,

    /// The spatially unique node identifier.
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
        guid_parse::try_parse_guid(s)
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
