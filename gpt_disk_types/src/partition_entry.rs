// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{
    guid, Guid, GuidFromStrError, LbaLe, LbaRangeInclusive, U16Le, U64Le,
};
use core::fmt::{self, Display, Formatter};
use core::num::NonZeroU32;
use core::str::FromStr;

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

/// Unique ID representing the type of a partition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(transparent)]
pub struct GptPartitionType(pub Guid);

// This lint incorrectly says that "ChromeOS" should be in backticks.
#[allow(clippy::doc_markdown)]
impl GptPartitionType {
    /// Indicates an entry within the GPT partition array is not in use.
    pub const UNUSED: Self = Self(Guid::ZERO);

    /// EFI system partition.
    ///
    /// This constant is defined in the UEFI Specification in Table 5-7
    /// "Defined GPT Partition Entry - Partition Type GUIDs".
    pub const EFI_SYSTEM: Self =
        Self(guid!("c12a7328-f81f-11d2-ba4b-00a0c93ec93b"));

    /// Partition containing a legacy MBR.
    ///
    /// This constant is defined in the UEFI Specification in Table 5-7
    /// "Defined GPT Partition Entry - Partition Type GUIDs".
    pub const LEGACY_MBR: Self =
        Self(guid!("024dee41-33e7-11d3-9d69-0008c781f39f"));

    /// Basic data partition.
    pub const BASIC_DATA: Self =
        Self(guid!("ebd0a0a2-b9e5-4433-87c0-68b6b72699c7"));

    /// ChromeOS kernel partition.
    pub const CHROME_OS_KERNEL: Self =
        Self(guid!("fe3a2a5d-4f32-41a7-b725-accc3285a309"));

    /// ChromeOS rootfs partition.
    pub const CHROME_OS_ROOT_FS: Self =
        Self(guid!("3cb8e202-3b7e-47dd-8a3c-7ff2a13cfcec"));

    // TODO: there are many more "known" partition types for which we
    // could add constants.
}

impl Display for GptPartitionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self == &Self::UNUSED {
            f.write_str("UNUSED")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl FromStr for GptPartitionType {
    type Err = GuidFromStrError;

    /// Parse from a GUID string. See [`Guid::from_str`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

/// Partition attribute bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(transparent)]
pub struct GptPartitionAttributes(pub U64Le);

impl GptPartitionAttributes {
    /// If set, bit `0` indicates the partition is required for the
    /// platform to function.
    pub const REQUIRED_PARTITION_BIT: u8 = 0;

    /// If set, bit `1` tells the firmware not to provide
    /// `EFI_BLOCK_IO_PROTOCOL` for this partition.
    pub const NO_BLOCK_IO_PROTOCOL_BIT: u8 = 1;

    /// If set, bit `2` indicates to specialized software on legacy BIOS
    /// systems that the partition may be bootable. This bit is ignored
    /// by UEFI boot loaders.
    pub const LEGACY_BIOS_BOOTABLE_BIT: u8 = 2;

    fn get_bit(self, bit: u8) -> bool {
        self.0 .0[0] & (1 << bit) != 0
    }

    fn set_bit(&mut self, bit: u8, set: bool) {
        if set {
            self.0 .0[0] |= 1 << bit;
        } else {
            self.0 .0[0] &= !(1 << bit);
        }
    }

    /// Get the [`REQUIRED_PARTITION_BIT`] attribute value.
    ///
    /// [`REQUIRED_PARTITION_BIT`]: Self::REQUIRED_PARTITION_BIT
    #[must_use]
    pub fn required_partition(self) -> bool {
        self.get_bit(Self::REQUIRED_PARTITION_BIT)
    }

    /// Update the [`REQUIRED_PARTITION_BIT`] attribute value.
    ///
    /// [`REQUIRED_PARTITION_BIT`]: Self::REQUIRED_PARTITION_BIT
    pub fn update_required_partition(&mut self, required: bool) {
        self.set_bit(Self::REQUIRED_PARTITION_BIT, required);
    }

    /// Get the [`NO_BLOCK_IO_PROTOCOL_BIT`] attribute value.
    ///
    /// [`NO_BLOCK_IO_PROTOCOL_BIT`]: Self::NO_BLOCK_IO_PROTOCOL_BIT
    #[must_use]
    pub fn no_block_io_protocol(self) -> bool {
        self.get_bit(Self::NO_BLOCK_IO_PROTOCOL_BIT)
    }

    /// Update the [`NO_BLOCK_IO_PROTOCOL_BIT`] attribute value.
    ///
    /// [`NO_BLOCK_IO_PROTOCOL_BIT`]: Self::NO_BLOCK_IO_PROTOCOL_BIT
    pub fn update_no_block_io_protocol(&mut self, no_block_io_protocol: bool) {
        self.set_bit(Self::NO_BLOCK_IO_PROTOCOL_BIT, no_block_io_protocol);
    }

    /// Get the [`LEGACY_BIOS_BOOTABLE_BIT`] attribute value.
    ///
    /// [`LEGACY_BIOS_BOOTABLE_BIT`]: Self::LEGACY_BIOS_BOOTABLE_BIT
    #[must_use]
    pub fn legacy_bios_bootable(self) -> bool {
        self.get_bit(Self::LEGACY_BIOS_BOOTABLE_BIT)
    }

    /// Update the [`LEGACY_BIOS_BOOTABLE_BIT`] attribute value.
    ///
    /// [`LEGACY_BIOS_BOOTABLE_BIT`]: Self::LEGACY_BIOS_BOOTABLE_BIT
    pub fn update_legacy_bios_bootable(&mut self, legacy_bios_bootable: bool) {
        self.set_bit(Self::LEGACY_BIOS_BOOTABLE_BIT, legacy_bios_bootable);
    }

    /// Bits `48..=63` represented as a [`U16Le`]. These bits are
    /// reserved for custom use by the partition type, so their meaning
    /// depends on [`GptPartitionEntry::partition_type_guid`].
    #[must_use]
    pub fn type_specific_attributes(self) -> U16Le {
        U16Le([self.0 .0[6], self.0 .0[7]])
    }

    /// Set bits `48..=63`. These bits are reserved for custom use by
    /// the partition type, so their meaning depends on
    /// [`GptPartitionEntry::partition_type_guid`].
    pub fn update_type_specific_attributes(&mut self, attrs: U16Le) {
        self.0 .0[6] = attrs.0[0];
        self.0 .0[7] = attrs.0[1];
    }
}

impl Display for GptPartitionAttributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let mut sep = |f: &mut Formatter<'_>| {
            if first {
                first = false;
            } else {
                f.write_str(", ")?;
            }
            Ok(())
        };

        if self.required_partition() {
            sep(f)?;
            f.write_str("required_partition (1)")?;
        }
        if self.no_block_io_protocol() {
            sep(f)?;
            f.write_str("no_block_io_protocol (2)")?;
        }
        if self.legacy_bios_bootable() {
            sep(f)?;
            f.write_str("legacy_bios_bootable (4)")?;
        }
        let type_specific = self.type_specific_attributes();
        if type_specific.to_u16() != 0 {
            sep(f)?;
            write!(f, "type_specific({:#x})", self.type_specific_attributes())?;
        }
        if first {
            write!(f, "(empty)")?;
        }
        Ok(())
    }
}

struct GptPartitionNameCharIter<'a> {
    name: &'a GptPartitionName,
    byte_index: usize,
}

impl Iterator for GptPartitionNameCharIter<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = &self.name.0;

        // Stop iteration at the end of the name.
        if self.byte_index >= bytes.len() {
            return None;
        }

        // UEFI strings are UCS-2, not UTF-16. That means that each
        // source character is exactly two bytes long.
        let c = (u16::from(bytes[self.byte_index + 1]) << 8)
            | u16::from(bytes[self.byte_index]);

        // Stop iteration at the first null terminator.
        if c == 0 {
            self.byte_index = bytes.len();
            return None;
        }

        self.byte_index += 2;

        Some(char::try_from(u32::from(c)).unwrap_or('�'))
    }
}

/// Error type for [`GptPartitionName::set_char`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GptPartitionNameSetCharError {
    /// Character index is outside the range `0..36`.
    Index,

    /// Character cannot be represented in UCS-2.
    InvalidChar,
}

impl Display for GptPartitionNameSetCharError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index => f.write_str("invalid character index"),
            Self::InvalidChar => {
                f.write_str("character cannot be represented in UCS-2")
            }
        }
    }
}

impl core::error::Error for GptPartitionNameSetCharError {}

/// Human readable partition label encoded as a null-terminated UCS-2
/// string.
///
/// # Examples
///
/// Construct from a UTF-8 string:
///
/// ```
/// use gpt_disk_types::GptPartitionName;
///
/// let partition_name: GptPartitionName = "hacktheplanet".parse().unwrap();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct GptPartitionName(pub [u8; 72]);

// Manual implementation needed because of the large array.
#[cfg(feature = "bytemuck")]
#[allow(unsafe_code)]
unsafe impl Pod for GptPartitionName {}
#[cfg(feature = "bytemuck")]
#[allow(unsafe_code)]
unsafe impl Zeroable for GptPartitionName {}

impl GptPartitionName {
    /// True if the first character is a null terminator, false otherwise.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0
    }

    /// Get an iterator over the characters in the partition name, using
    /// UCS-2 decoding. Iteration ends when either the end of the array
    /// or a null terminator is reached. The null character is not
    /// included in the iteration output. Any invalid characters are
    /// replaced with the Unicode replacement character (`�`).
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        GptPartitionNameCharIter {
            name: self,
            byte_index: 0,
        }
    }

    /// Set a UCS-2 character. The `index` is by UCS-2 character rather
    /// than byte (e.g. index 3 indicates byte offset 6). This is valid
    /// because UCS-2 is a fixed-width encoding.
    pub fn set_char(
        &mut self,
        index: usize,
        c: char,
    ) -> Result<(), GptPartitionNameSetCharError> {
        // Ensure the index is valid.
        if index > self.0.len() / 2 {
            return Err(GptPartitionNameSetCharError::Index);
        }

        let c = u16::try_from(u32::from(c))
            .map_err(|_| GptPartitionNameSetCharError::InvalidChar)?;
        let bytes = c.to_le_bytes();
        self.0[index * 2] = bytes[0];
        self.0[index * 2 + 1] = bytes[1];
        Ok(())
    }
}

impl Display for GptPartitionName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for c in self.chars() {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

impl Default for GptPartitionName {
    fn default() -> Self {
        Self([0; 72])
    }
}

/// Error type for [`GptPartitionName::from_str`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum GptPartitionNameFromStrError {
    /// Input string is too long.
    Length,

    /// Input string contains a character that cannot be represented in UCS-2.
    InvalidChar,
}

impl Display for GptPartitionNameFromStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length => f.write_str("input string is too long"),
            Self::InvalidChar => f.write_str("input string contains a character that cannot be represented in UCS-2"),
        }
    }
}

impl From<ucs2::Error> for GptPartitionNameFromStrError {
    fn from(err: ucs2::Error) -> Self {
        match err {
            ucs2::Error::BufferOverflow => Self::Length,
            ucs2::Error::MultiByte => Self::InvalidChar,
        }
    }
}

impl core::error::Error for GptPartitionNameFromStrError {}

impl FromStr for GptPartitionName {
    type Err = GptPartitionNameFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut name = Self::default();

        // Leave room for null terminator.
        let max_index = name.0.len() - 2 - 1;

        let mut index = 0;
        ucs2::encode_with(s, |c| {
            if index >= max_index {
                Err(ucs2::Error::BufferOverflow)
            } else {
                name.0[index] = u8::try_from(c & 0xff).unwrap();
                name.0[index + 1] = u8::try_from((c & 0xff00) >> 8).unwrap();
                index += 2;
                Ok(())
            }
        })?;
        Ok(name)
    }
}

/// An entry within the GPT partition array.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C, packed)]
pub struct GptPartitionEntry {
    /// Unique ID representing the partition's type. If the type is
    /// [`GptPartitionType::UNUSED`], this entry in the partition array
    /// is not in use.
    pub partition_type_guid: GptPartitionType,

    /// GUID that is unique for every partition entry.
    pub unique_partition_guid: Guid,

    /// LBA of the partition's first block.
    pub starting_lba: LbaLe,

    /// LBA of the partition's last block.
    pub ending_lba: LbaLe,

    /// Attribute bit flags.
    pub attributes: GptPartitionAttributes,

    /// Human readable partition label encoded as a null-terminated
    /// UCS-2 string.
    pub name: GptPartitionName,
}

impl GptPartitionEntry {
    /// Get the range of blocks covered by this partition. Returns
    /// `None` if the `ending_lba` is less than the `starting_lba`.
    #[must_use]
    pub fn lba_range(&self) -> Option<LbaRangeInclusive> {
        LbaRangeInclusive::new(self.starting_lba.into(), self.ending_lba.into())
    }

    /// Check if the entry is in use. If the [`partition_type_guid`] is
    /// [`GptPartitionType::UNUSED`], the entry is considered unused,
    /// which means there is no partition data associated with the entry.
    ///
    /// [`partition_type_guid`]: Self::partition_type_guid
    #[must_use]
    pub fn is_used(&self) -> bool {
        let partition_type_guid = self.partition_type_guid;
        partition_type_guid != GptPartitionType::UNUSED
    }
}

impl Display for GptPartitionEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("GptPartitionEntry { ")?;
        write!(f, "partition_type_guid: {}", &{ self.partition_type_guid })?;
        write!(f, ", unique_partition_guid: {}", &{
            self.unique_partition_guid
        })?;
        write!(f, ", starting_lba: {}", self.starting_lba)?;
        write!(f, ", ending_lba: {}", self.ending_lba)?;
        write!(f, ", attributes: {}", self.attributes)?;
        write!(f, ", name: \"{}\"", self.name)?;
        f.write_str(" }")
    }
}

/// Error returned by [`GptPartitionEntrySize::new`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct GptPartitionEntrySizeError;

impl Display for GptPartitionEntrySizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("partition entry size must be a power of two greater than or equal to 128")
    }
}

impl core::error::Error for GptPartitionEntrySizeError {}

/// Size in bytes of entries in the partition entry array.
///
/// A valid partition entry size must be a value of 128×2ⁿ, where n is
/// an integer ≥0.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct GptPartitionEntrySize(NonZeroU32);

impl GptPartitionEntrySize {
    /// Create a new `GptPartitionEntrySize`. Returns
    /// [`GptPartitionEntrySizeError`] if the input is less than 128 or
    /// not a power of two.
    pub const fn new(val: u32) -> Result<Self, GptPartitionEntrySizeError> {
        if let Some(nz) = NonZeroU32::new(val) {
            if val >= 128 && val.is_power_of_two() {
                Ok(Self(nz))
            } else {
                Err(GptPartitionEntrySizeError)
            }
        } else {
            Err(GptPartitionEntrySizeError)
        }
    }

    /// Get the entry size in bytes as a [`u32`].
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        self.0.get()
    }

    /// Get the entry size in bytes as a [`u64`].
    #[allow(clippy::as_conversions)]
    #[must_use]
    pub const fn to_u64(self) -> u64 {
        self.0.get() as u64
    }

    /// Get the entry size in bytes as a [`usize`].
    #[must_use]
    pub fn to_usize(self) -> Option<usize> {
        self.0.get().try_into().ok()
    }
}

impl Default for GptPartitionEntrySize {
    fn default() -> Self {
        Self::new(128).unwrap()
    }
}

impl Display for GptPartitionEntrySize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
