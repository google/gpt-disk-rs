// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    Crc32, GptPartitionEntry, GptPartitionEntryArrayLayout,
    GptPartitionEntrySize, GptPartitionEntrySizeError, Guid, LbaLe, U32Le,
    U64Le,
};
use core::fmt::{self, Display, Formatter};
use core::mem;

#[cfg(feature = "bytemuck")]
use bytemuck::{bytes_of, Pod, Zeroable};

/// GPT header signature.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(transparent)]
pub struct GptHeaderSignature(pub U64Le);

impl Display for GptHeaderSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Signature(")?;
        if *self == Self::EFI_COMPATIBLE_PARTITION_TABLE_HEADER {
            f.write_str("\"EFI PART\"")?;
        } else {
            write!(f, "Invalid: {:#016x}", self.0)?;
        }
        f.write_str(")")
    }
}

impl GptHeaderSignature {
    /// EFI-compatible partition table header. This is the only valid
    /// signature.
    pub const EFI_COMPATIBLE_PARTITION_TABLE_HEADER: Self =
        Self(U64Le(*b"EFI PART"));

    /// Convert to [`u64`] with the host's endianness.
    #[must_use]
    pub const fn to_u64(self) -> u64 {
        self.0.to_u64()
    }
}

impl Default for GptHeaderSignature {
    fn default() -> Self {
        Self::EFI_COMPATIBLE_PARTITION_TABLE_HEADER
    }
}

/// GPT header revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(transparent)]
pub struct GptHeaderRevision(pub U32Le);

impl GptHeaderRevision {
    /// Version 1.0. This is the only valid revision.
    pub const VERSION_1_0: Self = Self(U32Le::from_u32(0x0001_0000));

    /// Get the major part of the version.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn major(self) -> u16 {
        u16::from_le_bytes(self.0 .0[2..4].try_into().unwrap())
    }

    /// Get the minor part of the version.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn minor(self) -> u16 {
        u16::from_le_bytes(self.0 .0[0..2].try_into().unwrap())
    }
}

impl Default for GptHeaderRevision {
    fn default() -> Self {
        Self::VERSION_1_0
    }
}

impl Display for GptHeaderRevision {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#08x}", self.0)
    }
}

/// GPT header that appears near the start and end of a GPT-formatted disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C)]
pub struct GptHeader {
    /// Magic signature for the header. In a valid header this must be
    /// [`GptHeaderSignature::EFI_COMPATIBLE_PARTITION_TABLE_HEADER`].
    pub signature: GptHeaderSignature,

    /// Revision number for the header. In a valid header this must be
    /// [`GptHeaderRevision::VERSION_1_0`].
    pub revision: GptHeaderRevision,

    /// Size of the header in bytes. In a valid header this must be
    /// greater than or equal to 92 bytes, and less than or equal to the
    /// block size.
    pub header_size: U32Le,

    /// CRC32 checksum of the entire header. When calculating the
    /// checksum, this field is included in the checksum as four zero
    /// bytes.
    pub header_crc32: Crc32,

    /// Reserved bytes. In a valid header these must be all zero.
    pub reserved: U32Le,

    /// The LBA that contains this header.
    pub my_lba: LbaLe,

    /// The LBA that contains the alternate header.
    pub alternate_lba: LbaLe,

    /// First LBA that can be used for the data of a partition in the
    /// partition entry array.
    pub first_usable_lba: LbaLe,

    /// Last LBA that can be used for the data of a partition in the
    /// partition entry array.
    pub last_usable_lba: LbaLe,

    /// Unique ID for the disk.
    pub disk_guid: Guid,

    /// First LBA of the partition entry array.
    pub partition_entry_lba: LbaLe,

    /// Number of partitions in the partition entry array.
    pub number_of_partition_entries: U32Le,

    /// Size in bytes of each entry in the partition entry array.
    pub size_of_partition_entry: U32Le,

    /// CRC32 checksum of the partition entry array.
    pub partition_entry_array_crc32: Crc32,
}

impl GptHeader {
    /// Check if the header's signature matches
    /// [`GptHeaderSignature::EFI_COMPATIBLE_PARTITION_TABLE_HEADER`].
    #[must_use]
    pub fn is_signature_valid(&self) -> bool {
        self.signature
            == GptHeaderSignature::EFI_COMPATIBLE_PARTITION_TABLE_HEADER
    }

    /// Calculate the header's CRC32 checksum. This returns the checksum
    /// but does not update the checksum field in the header.
    #[cfg(feature = "bytemuck")]
    #[must_use]
    pub fn calculate_header_crc32(&self) -> Crc32 {
        let crc = crc::Crc::<u32>::new(&Crc32::ALGORITHM);
        let mut digest = crc.digest();
        digest.update(bytes_of(&self.signature));
        digest.update(bytes_of(&self.revision));
        digest.update(bytes_of(&self.header_size));
        digest.update(&[0u8; 4]); // Zeroes for the `header_crc32` field.
        digest.update(bytes_of(&self.reserved));
        digest.update(bytes_of(&self.my_lba));
        digest.update(bytes_of(&self.alternate_lba));
        digest.update(bytes_of(&self.first_usable_lba));
        digest.update(bytes_of(&self.last_usable_lba));
        digest.update(bytes_of(&self.disk_guid));
        digest.update(bytes_of(&self.partition_entry_lba));
        digest.update(bytes_of(&self.number_of_partition_entries));
        digest.update(bytes_of(&self.size_of_partition_entry));
        digest.update(bytes_of(&self.partition_entry_array_crc32));
        Crc32(U32Le(digest.finalize().to_le_bytes()))
    }

    /// Update the header's CRC32 checksum.
    #[cfg(feature = "bytemuck")]
    pub fn update_header_crc32(&mut self) {
        self.header_crc32 = self.calculate_header_crc32();
    }

    /// Get the [`GptPartitionEntryArrayLayout`] for this header.
    pub fn get_partition_entry_array_layout(
        &self,
    ) -> Result<GptPartitionEntryArrayLayout, GptPartitionEntrySizeError> {
        Ok(GptPartitionEntryArrayLayout {
            start_lba: self.partition_entry_lba.into(),
            entry_size: GptPartitionEntrySize::new(
                self.size_of_partition_entry.to_u32(),
            )?,
            num_entries: self.number_of_partition_entries.to_u32(),
        })
    }
}

impl Default for GptHeader {
    fn default() -> Self {
        Self {
            signature: GptHeaderSignature::default(),
            revision: GptHeaderRevision::default(),
            header_size: U32Le::from_u32(
                u32::try_from(mem::size_of::<Self>()).unwrap(),
            ),
            header_crc32: Crc32::default(),
            reserved: U32Le::default(),
            my_lba: LbaLe::default(),
            alternate_lba: LbaLe::default(),
            first_usable_lba: LbaLe::default(),
            last_usable_lba: LbaLe::default(),
            disk_guid: Guid::default(),
            partition_entry_lba: LbaLe::default(),
            number_of_partition_entries: U32Le::default(),
            size_of_partition_entry: U32Le::from_u32(
                u32::try_from(mem::size_of::<GptPartitionEntry>()).unwrap(),
            ),
            partition_entry_array_crc32: Crc32::default(),
        }
    }
}

impl Display for GptHeader {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "GptHeader {{ signature: {}", self.signature)?;
        write!(f, ", revision: {:#x}", self.revision.0)?;
        write!(f, ", header_size: {}", self.header_size.to_u32())?;
        write!(f, ", header_crc32: {:#x}", self.header_crc32)?;
        write!(f, ", my_lba: {}", self.my_lba)?;
        write!(f, ", alternate_lba: {}", self.alternate_lba)?;
        write!(f, ", first_usable_lba: {}", self.first_usable_lba)?;
        write!(f, ", last_usable_lba: {}", self.last_usable_lba)?;
        write!(f, ", disk_guid: {}", self.disk_guid)?;
        write!(f, ", partition_entry_lba: {}", self.partition_entry_lba)?;
        write!(
            f,
            ", number_of_partition_entries: {}",
            self.number_of_partition_entries
        )?;
        write!(
            f,
            ", size_of_partition_entry: {}",
            self.size_of_partition_entry
        )?;
        write!(
            f,
            ", partition_entry_array_crc32: {:#x}",
            self.partition_entry_array_crc32
        )?;
        f.write_str(" }")
    }
}
