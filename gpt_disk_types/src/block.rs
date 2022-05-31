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

use crate::U64Le;
use bytemuck::{Pod, Zeroable};
use core::fmt::{self, Display, Formatter};
use core::num::{NonZeroU32, TryFromIntError};
use core::ops::RangeInclusive;

/// Logical block address.
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
pub struct Lba(pub u64);

impl Lba {
    /// Convert to a plain [`u64`].
    #[must_use]
    pub fn to_u64(self) -> u64 {
        self.0
    }
}

impl PartialEq<u64> for Lba {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl Display for Lba {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<Lba> for usize {
    type Error = TryFromIntError;

    fn try_from(lba: Lba) -> Result<Self, Self::Error> {
        lba.0.try_into()
    }
}

impl From<LbaLe> for Lba {
    fn from(lba: LbaLe) -> Self {
        Self(lba.to_u64())
    }
}

/// Logical block address stored as a [`U64Le`].
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
pub struct LbaLe(pub U64Le);

impl LbaLe {
    /// Create a logical block address from a [`u64`].
    #[must_use]
    pub const fn from_u64(v: u64) -> Self {
        Self(U64Le::from_u64(v))
    }

    /// Get the logical block address as a [`u64`].
    #[must_use]
    pub const fn to_u64(self) -> u64 {
        self.0.to_u64()
    }
}

impl Display for LbaLe {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_u64().fmt(f)
    }
}

impl From<Lba> for LbaLe {
    fn from(lba: Lba) -> Self {
        Self::from_u64(lba.0)
    }
}

/// Inclusive range of logical block addresses.
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
#[repr(C)]
pub struct LbaRangeInclusive {
    start: Lba,
    end: Lba,
}

impl LbaRangeInclusive {
    /// Create an LBA range. The end address must be greater than or
    /// equal to the start address.
    #[must_use]
    pub const fn new(start: Lba, end: Lba) -> Option<LbaRangeInclusive> {
        if end.0 >= start.0 {
            Some(LbaRangeInclusive { start, end })
        } else {
            None
        }
    }

    /// Starting LBA (inclusive).
    #[must_use]
    pub const fn start(self) -> Lba {
        self.start
    }

    /// Ending LBA (inclusive).
    #[must_use]
    pub const fn end(self) -> Lba {
        self.end
    }

    /// Create an LBA range from the corresponding byte range for the
    /// given block size.
    ///
    /// The byte range must correspond precisely to block bounds, with
    /// the start byte at the beginning of a block and the end byte at
    /// the end of a block
    ///
    /// # Examples
    ///
    /// ```
    /// use gpt_disk_types::{BlockSize, LbaRangeInclusive};
    ///
    /// # fn t() -> Option<()> {
    /// let block_size = BlockSize::new(4)?;
    /// let r = LbaRangeInclusive::from_byte_range(4..=11, block_size)?;
    /// assert_eq!(r.start().0, 1);
    /// assert_eq!(r.end().0, 2);
    /// # None
    /// # }
    /// #
    /// # fn main() { t(); }
    #[must_use]
    pub fn from_byte_range(
        byte_range: RangeInclusive<u64>,
        block_size: BlockSize,
    ) -> Option<Self> {
        let start_byte = byte_range.start();
        let end_byte_plus_1 = byte_range.end().checked_add(1)?;
        let block_size = block_size.to_u64();

        if (start_byte % block_size) != 0 {
            return None;
        }
        if (end_byte_plus_1 % block_size) != 0 {
            return None;
        }

        let end_lba = (end_byte_plus_1 / block_size).checked_sub(1)?;

        LbaRangeInclusive::new(Lba(start_byte / block_size), Lba(end_lba))
    }

    /// Convert the LBA range to the corresponding byte range for the
    /// given block size.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpt_disk_types::{BlockSize, Lba, LbaRangeInclusive};
    ///
    /// # fn t() -> Option<()> {
    /// let r = LbaRangeInclusive::new(Lba(1), Lba(2))?;
    /// let block_size = BlockSize::new(4)?;
    /// assert_eq!(r.to_byte_range(block_size)?, 4..=11);
    /// # None
    /// # }
    /// #
    /// # fn main() { t(); }
    /// ```
    #[must_use]
    pub fn to_byte_range(
        self,
        block_size: BlockSize,
    ) -> Option<RangeInclusive<u64>> {
        let block_size = block_size.to_u64();
        let start_byte = self.start.0.checked_mul(block_size)?;
        let end_byte = self
            .end
            .0
            .checked_mul(block_size)?
            .checked_add(block_size - 1)?;
        Some(start_byte..=end_byte)
    }
}

impl Display for LbaRangeInclusive {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}..={}", self.start, self.end)
    }
}

/// Size of a block in bytes.
///
/// This type enforces some restrictions on the block size: it must be
/// at least 512 bytes and fit within a [`u32`].
///
/// # Minimum size
///
/// The [`MasterBootRecord`] size is 512 bytes and must fit within a
/// block, so the block size must be at least that large.
///
/// [`MasterBootRecord`]: crate::MasterBootRecord
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct BlockSize(NonZeroU32);

impl BlockSize {
    /// 512-byte block size.
    pub const B512: Self = Self(if let Some(nz) = NonZeroU32::new(512) {
        nz
    } else {
        unreachable!()
    });

    /// 4096-byte block size.
    pub const B4096: Self = Self(if let Some(nz) = NonZeroU32::new(4096) {
        nz
    } else {
        unreachable!()
    });

    /// Create a `BlockSize`.
    #[must_use]
    pub const fn new(num_bytes: u32) -> Option<Self> {
        if let Some(nz) = NonZeroU32::new(num_bytes) {
            if num_bytes >= 512 {
                Some(Self(nz))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Create a `BlockSize`.
    #[must_use]
    pub fn from_usize(num_bytes: usize) -> Option<Self> {
        Self::new(u32::try_from(num_bytes).ok()?)
    }

    /// Get the size in bytes as a [`u32`].
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        self.0.get()
    }

    /// Get the size in bytes as a [`u64`].
    #[allow(clippy::as_conversions)]
    #[must_use]
    pub const fn to_u64(self) -> u64 {
        self.0.get() as u64
    }

    /// Get the size in bytes as a [`usize`].
    #[must_use]
    pub fn to_usize(self) -> Option<usize> {
        self.0.get().try_into().ok()
    }
}

impl Default for BlockSize {
    fn default() -> Self {
        BlockSize::B512
    }
}

impl Display for BlockSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
