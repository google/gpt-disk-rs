// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::U32Le;
use bytemuck::{Pod, Zeroable};
use core::fmt::{self, Debug, Display, Formatter, LowerHex};

/// 32-bit CRC (cyclic redundency check).
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
    Pod,
    Zeroable,
)]
#[repr(transparent)]
pub struct Crc32(pub U32Le);

// It would be good to replace the long-winded notes above
// `Crc32::ALGORITHM` with something simpler and/or more authoritative
// if possible.

impl Crc32 {
    /// CRC32 algorithm used for GPT: [`crc::CRC_32_ISO_HDLC`]
    ///
    /// # Notes
    ///
    /// The UEFI Specification is somewhat vague about the CRC algorithm
    /// used. Section 4.2 EFI Table Header says: "Unless otherwise
    /// specified, UEFI uses a standard CCITT32 CRC algorithm with a
    /// seed polynomial value of 0x04c11db7 for its CRC calculations."
    /// There are no further mentions of either "CCITT32" or "04c11db7"
    /// in the spec. It's not clear what if any specification CCITT32
    /// refers to.
    ///
    /// The [Catalogue of parametrised CRC algorithms], which is the
    /// source of truth for the catalog used by the `crc` crate, has no
    /// references to CCITT32, but does have several entries that use
    /// the `0x04c11db7` polynomial. Of these, CRC-32/ISO-HDLC appears
    /// to be widely used and recommended by ITU-T, which is the
    /// successor of CCITT.
    ///
    /// [Catalogue of parametrised CRC algorithms]: https://reveng.sourceforge.io/crc-catalogue/17plus.htm
    pub const ALGORITHM: crc::Algorithm<u32> = crc::CRC_32_ISO_HDLC;
}

impl Display for Crc32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:#x}")
    }
}

impl LowerHex for Crc32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}
