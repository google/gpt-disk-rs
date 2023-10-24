// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::util::{byte_to_ascii_hex_lower, parse_byte_from_ascii_str_at};
use crate::GuidFromStrError;
use core::fmt::{self, Display, Formatter};
use core::str::{self, FromStr};

#[cfg(feature = "serde")]
use {
    serde::de::{self, Visitor},
    serde::{Deserialize, Deserializer, Serialize, Serializer},
};

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

/// Globally-unique identifier.
///
/// The format is defined in [RFC 4122]. However, unlike "normal" UUIDs
/// (such as those provided by the [`uuid`] crate), the first three
/// fields are little-endian. See also [Appendix A] of the UEFI
/// Specification.
///
/// This type is 4-byte aligned. The UEFI Specification says the GUID
/// type should be 8-byte aligned, but most C implementations have
/// 4-byte alignment, so we do the same here for compatibility.
///
/// [Appendix A]: https://uefi.org/specs/UEFI/2.10/Apx_A_GUID_and_Time_Formats.html
/// [RFC 4122]: https://datatracker.ietf.org/doc/html/rfc4122
/// [`uuid`]: https://docs.rs/uuid/latest/uuid
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C)]
pub struct Guid {
    // Use `u32` rather than `[u8; 4]` here so that the natural
    // alignment of the struct is four bytes. This is better for the end
    // user than setting `repr(align(4))` because it doesn't prevent use
    // of the type in a `repr(packed)` struct. For more discussion, see
    // https://github.com/rust-lang/rfcs/pull/1358#issuecomment-217582887
    time_low: u32,
    // For consistency with the above field, use `u16` for these fields.
    time_mid: u16,
    time_high_and_version: u16,
    clock_seq_high_and_reserved: u8,
    clock_seq_low: u8,
    node: [u8; 6],
}

impl Guid {
    /// GUID with all fields set to zero.
    pub const ZERO: Self = Self {
        time_low: 0,
        time_mid: 0,
        time_high_and_version: 0,
        clock_seq_high_and_reserved: 0,
        clock_seq_low: 0,
        node: [0; 6],
    };

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
            time_low: u32::from_le_bytes([
                time_low[0],
                time_low[1],
                time_low[2],
                time_low[3],
            ]),
            time_mid: u16::from_le_bytes([time_mid[0], time_mid[1]]),
            time_high_and_version: u16::from_le_bytes([
                time_high_and_version[0],
                time_high_and_version[1],
            ]),
            clock_seq_high_and_reserved,
            clock_seq_low,
            node,
        }
    }

    /// The little-endian low field of the timestamp.
    #[must_use]
    pub const fn time_low(self) -> [u8; 4] {
        self.time_low.to_le_bytes()
    }

    /// The little-endian middle field of the timestamp.
    #[must_use]
    pub const fn time_mid(self) -> [u8; 2] {
        self.time_mid.to_le_bytes()
    }

    /// The little-endian high field of the timestamp multiplexed with
    /// the version number.
    #[must_use]
    pub const fn time_high_and_version(self) -> [u8; 2] {
        self.time_high_and_version.to_le_bytes()
    }

    /// The high field of the clock sequence multiplexed with the
    /// variant.
    #[must_use]
    pub const fn clock_seq_high_and_reserved(self) -> u8 {
        self.clock_seq_high_and_reserved
    }

    /// The low field of the clock sequence.
    #[must_use]
    pub const fn clock_seq_low(self) -> u8 {
        self.clock_seq_low
    }

    /// The spatially unique node identifier.
    #[must_use]
    pub const fn node(self) -> [u8; 6] {
        self.node
    }

    /// Parse a GUID from a string.
    ///
    /// This is functionally the same as [`Self::from_str`], but is
    /// exposed separately to provide a `const` method for parsing.
    pub const fn try_parse(s: &str) -> Result<Self, GuidFromStrError> {
        // Treat input as ASCII.
        let s = s.as_bytes();

        if s.len() != 36 {
            return Err(GuidFromStrError::Length);
        }

        let sep = b'-';
        if s[8] != sep {
            return Err(GuidFromStrError::Separator(8));
        }
        if s[13] != sep {
            return Err(GuidFromStrError::Separator(13));
        }
        if s[18] != sep {
            return Err(GuidFromStrError::Separator(18));
        }
        if s[23] != sep {
            return Err(GuidFromStrError::Separator(23));
        }

        Ok(Self {
            time_low: u32::from_le_bytes([
                mtry!(parse_byte_from_ascii_str_at(s, 6)),
                mtry!(parse_byte_from_ascii_str_at(s, 4)),
                mtry!(parse_byte_from_ascii_str_at(s, 2)),
                mtry!(parse_byte_from_ascii_str_at(s, 0)),
            ]),
            time_mid: u16::from_le_bytes([
                mtry!(parse_byte_from_ascii_str_at(s, 11)),
                mtry!(parse_byte_from_ascii_str_at(s, 9)),
            ]),
            time_high_and_version: u16::from_le_bytes([
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

    /// Parse a GUID from a string, panicking on failure.
    ///
    /// The input must be in "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    /// format, where each `x` is a hex digit (any of `0-9`, `a-f`, or
    /// `A-F`).
    ///
    /// This function is marked `track_caller` so that error messages
    /// point directly to the invalid GUID string.
    ///
    /// # Panics
    ///
    /// This function will panic if the input is not in the format shown
    /// above. In particular, it will panic if the input is not exactly
    /// 36 bytes long, or if the input does not have separators at the
    /// expected positions, or if any of the remaining characters are
    /// not valid hex digits.
    #[must_use]
    #[track_caller]
    pub const fn parse_or_panic(s: &str) -> Self {
        match Self::try_parse(s) {
            Ok(g) => g,
            Err(GuidFromStrError::Length) => {
                panic!("GUID string has wrong length (expected 36 bytes)");
            }
            Err(GuidFromStrError::Separator(_)) => {
                panic!("GUID string is missing one or more separators (`-`)");
            }
            Err(GuidFromStrError::Hex(_)) => {
                panic!("GUID string contains one or more invalid characters");
            }
        }
    }

    /// Create a GUID from a 16-byte array. No changes to byte order are made.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            time_low: u32::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3],
            ]),
            time_mid: u16::from_le_bytes([bytes[4], bytes[5]]),
            time_high_and_version: u16::from_le_bytes([bytes[6], bytes[7]]),
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
        let time_low = self.time_low.to_le_bytes();
        let time_mid = self.time_mid.to_le_bytes();
        let time_high_and_version = self.time_high_and_version.to_le_bytes();

        [
            time_low[0],
            time_low[1],
            time_low[2],
            time_low[3],
            time_mid[0],
            time_mid[1],
            time_high_and_version[0],
            time_high_and_version[1],
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
        let time_low = self.time_low.to_le_bytes();
        let time_mid = self.time_mid.to_le_bytes();
        let time_high_and_version = self.time_high_and_version.to_le_bytes();

        let mut buf = [0; 36];
        (buf[0], buf[1]) = byte_to_ascii_hex_lower(time_low[3]);
        (buf[2], buf[3]) = byte_to_ascii_hex_lower(time_low[2]);
        (buf[4], buf[5]) = byte_to_ascii_hex_lower(time_low[1]);
        (buf[6], buf[7]) = byte_to_ascii_hex_lower(time_low[0]);
        buf[8] = b'-';
        (buf[9], buf[10]) = byte_to_ascii_hex_lower(time_mid[1]);
        (buf[11], buf[12]) = byte_to_ascii_hex_lower(time_mid[0]);
        buf[13] = b'-';
        (buf[14], buf[15]) = byte_to_ascii_hex_lower(time_high_and_version[1]);
        (buf[16], buf[17]) = byte_to_ascii_hex_lower(time_high_and_version[0]);
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

    /// Parse a GUID from a string, panicking on failure.
    ///
    /// The input must be in "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
    /// format, where each `x` is a hex digit (any of `0-9`, `a-f`, or
    /// `A-F`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_parse(s)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ascii = self.to_ascii_hex_lower();
        // OK to unwrap since the ascii output is valid utf-8.
        let s = str::from_utf8(&ascii).unwrap();
        serializer.serialize_str(s)
    }
}

#[cfg(feature = "serde")]
struct DeserializerVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for DeserializerVisitor {
    type Value = Guid;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a string in the format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Guid::try_parse(value).map_err(E::custom)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DeserializerVisitor)
    }
}
