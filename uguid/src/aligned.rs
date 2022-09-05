// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! TODO

use crate::util::{byte_to_ascii_hex_lower, parse_byte_from_ascii_str_at};
use crate::GuidFromStrError;
use core::fmt::{self, Display, Formatter};
use core::str::{self, FromStr};

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

/// Globally-unique identifier.
///
/// The format is described in Appendix A of the UEFI
/// Specification. Note that the first three fields are little-endian.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C, align(8))]
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

include!("guid_impl.rs");
