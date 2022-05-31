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

use crate::{
    BlockSize, Crc32, GptPartitionEntry, GptPartitionEntrySize, Lba, U32Le,
};
use bytemuck::{from_bytes, from_bytes_mut};
use core::fmt::{self, Display, Formatter};
use core::mem;
use core::ops::Range;

/// Disk layout of a GPT partition entry array.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GptPartitionEntryArrayLayout {
    /// First block of the array.
    pub start_lba: Lba,

    /// Size in bytes of each entry.
    pub entry_size: GptPartitionEntrySize,

    /// Number of entries in the array.
    pub num_entries: u32,
}

impl GptPartitionEntryArrayLayout {
    /// Get the number of blocks needed for this layout. Returns `None`
    /// if overflow occurs.
    #[must_use]
    pub fn num_blocks(&self, block_size: BlockSize) -> Option<u64> {
        let block_size = block_size.to_u64();
        let num_bytes_exact = self.num_bytes_exact()?;

        let mut num_blocks = num_bytes_exact / block_size;
        if num_bytes_exact % block_size != 0 {
            num_blocks = num_blocks.checked_add(1)?;
        }

        Some(num_blocks)
    }

    /// Get the number of blocks needed for this layout. Returns `None`
    /// if overflow occurs.
    #[must_use]
    pub fn num_blocks_as_usize(&self, block_size: BlockSize) -> Option<usize> {
        self.num_blocks(block_size)?.try_into().ok()
    }

    /// Get the number of bytes needed for the entries in this layout,
    /// ignoring any padding needed at the end to match the block
    /// size. This corresponds to the number of bytes that are covered
    /// by the [`partition_entry_array_crc32`].
    ///
    /// Returns `None` if overflow occurs.
    ///
    /// [`partition_entry_array_crc32`]: crate::GptHeader::partition_entry_array_crc32
    #[must_use]
    pub fn num_bytes_exact(&self) -> Option<u64> {
        let entry_size = self.entry_size.to_u64();
        let num_entries = u64::from(self.num_entries);
        entry_size.checked_mul(num_entries)
    }

    /// Get the number of bytes needed for the entries in this layout,
    /// ignoring any padding needed at the end to match the block
    /// size. This corresponds to the number of bytes that are covered
    /// by the [`partition_entry_array_crc32`].
    ///
    /// Returns `None` if overflow occurs.
    ///
    /// [`partition_entry_array_crc32`]: crate::GptHeader::partition_entry_array_crc32
    #[must_use]
    pub fn num_bytes_exact_as_usize(&self) -> Option<usize> {
        self.num_bytes_exact()?.try_into().ok()
    }

    /// Get the number of bytes needed for this layout, rounded up to
    /// the nearest block. This is equivalent to [`num_blocks`] *
    /// `block_size`.
    ///
    /// Returns `None` if overflow occurs.
    ///
    /// [`num_blocks`]: Self::num_blocks
    #[must_use]
    pub fn num_bytes_rounded_to_block(
        &self,
        block_size: BlockSize,
    ) -> Option<u64> {
        let num_blocks = self.num_blocks(block_size)?;
        num_blocks.checked_mul(block_size.to_u64())
    }

    /// Get the number of bytes needed for this layout, rounded up to
    /// the nearest block. This is equivalent to [`num_blocks`] *
    /// `block_size`.
    ///
    /// Returns `None` if overflow occurs.
    ///
    /// [`num_blocks`]: Self::num_blocks
    #[must_use]
    pub fn num_bytes_rounded_to_block_as_usize(
        &self,
        block_size: BlockSize,
    ) -> Option<usize> {
        self.num_bytes_rounded_to_block(block_size)?.try_into().ok()
    }
}

impl Display for GptPartitionEntryArrayLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "start_lba={}/entry_size={}/num_entries={}",
            self.start_lba, self.entry_size, self.num_entries
        )
    }
}

/// Errors used by [`GptPartitionEntryArray`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GptPartitionEntryArrayError {
    /// The storage buffer is not large enough. It must be at least
    /// [`layout.num_bytes_rounded_to_block`] in size.
    ///
    /// [`layout.num_bytes_rounded_to_block`]: GptPartitionEntryArrayLayout::num_bytes_rounded_to_block
    BufferTooSmall,

    /// Numeric overflow occurred.
    Overflow,
}

impl Display for GptPartitionEntryArrayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooSmall => f.write_str("storage buffer is too small"),
            Self::Overflow => f.write_str("numeric overflow occurred"),
        }
    }
}

/// Storage for a GPT partition entry array.
#[allow(missing_debug_implementations)]
pub struct GptPartitionEntryArray<'a> {
    layout: GptPartitionEntryArrayLayout,
    num_bytes_exact: usize,
    storage: &'a mut [u8],
}

impl<'a> GptPartitionEntryArray<'a> {
    /// Create a new `GptPartitionEntryArray` with the given
    /// `layout`. The length of `storage` must be at least
    /// [`layout.num_bytes_rounded_to_block`].
    ///
    /// [`layout.num_bytes_rounded_to_block`]: GptPartitionEntryArrayLayout::num_bytes_rounded_to_block
    pub fn new(
        layout: GptPartitionEntryArrayLayout,
        block_size: BlockSize,
        storage: &'a mut [u8],
    ) -> Result<Self, GptPartitionEntryArrayError> {
        let num_bytes_required = layout
            .num_bytes_rounded_to_block_as_usize(block_size)
            .ok_or(GptPartitionEntryArrayError::Overflow)?;

        let num_bytes_exact = layout
            .num_bytes_exact_as_usize()
            .ok_or(GptPartitionEntryArrayError::Overflow)?;

        let storage = storage
            .get_mut(..num_bytes_required)
            .ok_or(GptPartitionEntryArrayError::BufferTooSmall)?;

        Ok(Self {
            layout,
            num_bytes_exact,
            storage,
        })
    }

    /// Get a reference to the storage buffer.
    #[must_use]
    pub fn storage(&self) -> &[u8] {
        self.storage
    }

    /// Get a mutable reference to the storage buffer.
    #[must_use]
    pub fn storage_mut(&mut self) -> &mut [u8] {
        self.storage
    }

    /// Get the partition entry array layout.
    #[must_use]
    pub fn layout(&self) -> &GptPartitionEntryArrayLayout {
        &self.layout
    }

    /// Change the partition entry array's start [`Lba`].
    pub fn set_start_lba(&mut self, start_lba: Lba) {
        self.layout.start_lba = start_lba;
    }

    fn get_entry_byte_range(&self, index: u32) -> Option<Range<usize>> {
        if index >= self.layout.num_entries {
            return None;
        }

        let start = usize::try_from(
            u64::from(index) * u64::from(self.layout.entry_size.to_u32()),
        )
        .ok()?;
        Some(start..start + mem::size_of::<GptPartitionEntry>())
    }

    /// Get a partition entry reference. The `index` is zero-based.
    #[must_use]
    pub fn get_partition_entry(
        &self,
        index: u32,
    ) -> Option<&GptPartitionEntry> {
        Some(from_bytes(&self.storage[self.get_entry_byte_range(index)?]))
    }

    /// Get a mutable partition entry reference. The `index` is zero-based.
    #[must_use]
    pub fn get_partition_entry_mut(
        &mut self,
        index: u32,
    ) -> Option<&mut GptPartitionEntry> {
        let range = self.get_entry_byte_range(index)?;
        Some(from_bytes_mut(&mut self.storage[range]))
    }

    /// Calculate the CRC32 checksum for the partition entry array. The
    /// return value can then be set in the
    /// [`GptHeader::partition_entry_array_crc32`] field.
    ///
    /// [`GptHeader::partition_entry_array_crc32`]: crate::GptHeader::partition_entry_array_crc32
    #[must_use]
    pub fn calculate_crc32(&self) -> Crc32 {
        let crc = crc::Crc::<u32>::new(&Crc32::ALGORITHM);
        let mut digest = crc.digest();
        digest.update(&self.storage[..self.num_bytes_exact]);
        Crc32(U32Le(digest.finalize().to_le_bytes()))
    }
}
