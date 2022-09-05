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

/// Define a GUID struct and its impls. This is used to create structs
/// that are the same but for alignment.
macro_rules! guid_impl {
    (
        // Name of the GUID struct.
        $struct_name:ident,
        // Struct alignment.
        $struct_alignment:literal,
        // Name of the other GUID struct, used for From conversions.
        $other_struct_name:ident,
        // Internal name of the struct for implementing deserialization.
        $deserializer_name:ident,
        // Struct docstring.
        $struct_doc:literal) => {
        #[doc = $struct_doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
        #[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
        #[repr(C)]
        #[repr(align($struct_alignment))]
        pub struct $struct_name {
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

        impl $struct_name {
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

        impl Default for $struct_name {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl Display for $struct_name {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                let ascii = self.to_ascii_hex_lower();
                // OK to unwrap since the ascii output is valid utf-8.
                let s = str::from_utf8(&ascii).unwrap();
                f.write_str(s)
            }
        }

        impl FromStr for $struct_name {
            type Err = GuidFromStrError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_parse(s)
            }
        }

        impl From<$other_struct_name> for $struct_name {
            fn from(g: $other_struct_name) -> Self {
                Self {
                    time_low: g.time_low,
                    time_mid: g.time_mid,
                    time_high_and_version: g.time_high_and_version,
                    clock_seq_high_and_reserved: g.clock_seq_high_and_reserved,
                    clock_seq_low: g.clock_seq_low,
                    node: g.node,
                }
            }
        }

        #[cfg(feature = "serde")]
        impl Serialize for $struct_name {
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
        struct $deserializer_name;

        #[cfg(feature = "serde")]
        impl<'de> Visitor<'de> for $deserializer_name {
            type Value = $struct_name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string in the format \"xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx\"",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                $struct_name::try_parse(value).map_err(E::custom)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_str($deserializer_name)
            }
        }
    }
}

guid_impl!(
    Guid,
    1,
    AlignedGuid,
    GuidDeserializeVisitor,
    "Globally-unique identifier (1-byte aligned).

The format is described in Appendix A of the UEFI
Specification. Note that the first three fields are little-endian."
);

guid_impl!(
    AlignedGuid,
    8,
    Guid,
    AlignedGuidDeserializeVisitor,
    "Globally-unique identifier (8-byte aligned).

The format is described in Appendix A of the UEFI
Specification. Note that the first three fields are little-endian.

This type is compatible with the `EFI_GUID` type, which is specified
to be 8-byte aligned."
);
