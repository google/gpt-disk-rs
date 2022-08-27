// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::num::format_u8_slice_lower_hex_le;
use crate::{Lba, U32Le};
use core::fmt::{self, Display, Formatter};

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

/// Legacy disk geometry used for converting between [`Lba`] and [`Chs`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DiskGeometry {
    /// Heads per cylinder.
    pub heads_per_cylinder: u32,

    /// Sectors per track.
    pub sectors_per_track: u32,
}

impl DiskGeometry {
    /// These are the same fallback values that gdisk uses when the disk
    /// geometry isn't known.
    pub const UNKNOWN: Self = Self {
        heads_per_cylinder: 255,
        sectors_per_track: 63,
    };
}

impl Default for DiskGeometry {
    fn default() -> Self {
        Self::UNKNOWN
    }
}

impl Display for DiskGeometry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HPC={}/SPT={}",
            self.heads_per_cylinder, self.sectors_per_track
        )
    }
}

/// Legacy MBR cylinder/head/sector.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C)]
pub struct Chs(pub [u8; 3]);

impl Chs {
    /// Get the 10 cylinder bits as a [`u16`].
    #[must_use]
    pub fn cylinder(self) -> u16 {
        let h = self.0[1] & 0b1100_0000;
        let l = self.0[2];
        (u16::from(h) << 2) | u16::from(l)
    }

    /// Get the 8 head bits as a [`u8`].
    #[must_use]
    pub fn head(self) -> u8 {
        self.0[0]
    }

    /// Get the 6 sector bits as a [`u8`].
    #[must_use]
    pub fn sector(self) -> u8 {
        self.0[1] & 0b0011_1111
    }

    /// Get a tuple of `(cylinder, head, sector)`.
    #[must_use]
    pub fn as_tuple(self) -> (u16, u8, u8) {
        (self.cylinder(), self.head(), self.sector())
    }

    /// Create a new `Chs`. Returns `None` if `cylinder` can't fit in 10
    /// bits, or if `sector` can't fit in 6 bits.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(cylinder: u16, head: u8, sector: u8) -> Option<Self> {
        if (cylinder & 0b1111_1100_0000_0000) != 0 {
            return None;
        }
        if (sector & 0b1100_0000) != 0 {
            return None;
        }
        Some(Chs([
            head,
            u8::try_from((cylinder & 0b11_0000_0000) >> 2).unwrap()
                | (sector & 0b0011_1111),
            u8::try_from(cylinder & 0xff).unwrap(),
        ]))
    }

    /// Convert LBA to CHS address. Returns `None` if the LBA value
    /// cannot fit in the CHS format.
    #[must_use]
    pub fn from_lba(lba: Lba, geom: DiskGeometry) -> Option<Self> {
        let lba = u32::try_from(lba.0).ok()?;

        // https://en.wikipedia.org/wiki/Logical_block_addressing
        let cylinder = lba / (geom.heads_per_cylinder * geom.sectors_per_track);
        let head = (lba / geom.sectors_per_track) % geom.heads_per_cylinder;
        let sector = (lba % geom.sectors_per_track) + 1;

        Self::new(
            cylinder.try_into().ok()?,
            head.try_into().ok()?,
            sector.try_into().ok()?,
        )
    }
}

impl Display for Chs {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CHS={}/{}/{}",
            self.cylinder(),
            self.head(),
            self.sector()
        )
    }
}

/// Legacy MBR partition record.
///
/// See Table 5-2 "Legacy MBR Partition Record" in the UEFI Specification.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C)]
pub struct MbrPartitionRecord {
    /// A value of `0x80` indicates this is a legacy bootable
    /// partition. Any other value indicates it is not bootable. UEFI
    /// firmware does not use this field's value.
    pub boot_indicator: u8,

    /// Start of the partition. UEFI firmware does not use this field's
    /// value.
    pub start_chs: Chs,

    /// Type of partition. A value of `0xef` defines a system
    /// partition. A value of `0xee` is used in a protective MBR to
    /// define a fake partition covering the entire disk. Other values
    /// are possible but not defined by the UEFI Specification.
    ///
    /// See section 5.2.2 "OS Types" in the UEFI Specification.
    pub os_indicator: u8,

    /// End of the partition. UEFI firmware does not use this field's
    /// value.
    pub end_chs: Chs,

    /// Starting LBA of the partition. UEFI firmware uses this field to
    /// determine the start of the partition.
    pub starting_lba: U32Le,

    /// Size of the partition in logical blocks. UEFI firmware uses this
    /// field to determine the size of the partition.
    pub size_in_lba: U32Le,
}

impl Display for MbrPartitionRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("MbrPartitionRecord { ")?;
        write!(f, "boot_indicator: {:#x}", self.boot_indicator)?;
        write!(f, ", start_chs: {}", self.start_chs)?;
        write!(f, ", os_indicator: {:#x}", self.os_indicator)?;
        write!(f, ", end_chs: {}", self.end_chs)?;
        write!(f, ", starting_lba: {}", self.starting_lba)?;
        write!(f, ", size_in_lba: {}", self.size_in_lba)?;
        f.write_str(" }")
    }
}

/// Legacy master boot record.
///
/// See Table 5-1 "Legacy MBR" in the UEFI Specification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(C)]
pub struct MasterBootRecord {
    /// Executable code used on non-UEFI systems select a partition and
    /// load the first logical block of that partition.
    pub boot_strap_code: [u8; 440],

    /// Unique identifier for the disk. This value is not used by UEFI
    /// firmware.
    pub unique_mbr_disk_signature: [u8; 4],

    /// Reserved field that is not used by UEFI firmware.
    pub unknown: [u8; 2],

    /// Four legacy MBR partitions.
    pub partitions: [MbrPartitionRecord; 4],

    /// MBR signature, set to `0xaa55`.
    pub signature: [u8; 2],
}

// Manual implementation needed because of the large boot_strap_code
// array field.
impl Default for MasterBootRecord {
    fn default() -> Self {
        Self {
            boot_strap_code: [0; 440],
            unique_mbr_disk_signature: [0; 4],
            unknown: [0, 2],
            partitions: [MbrPartitionRecord::default(); 4],
            signature: [0; 2],
        }
    }
}

// Manual implementation needed because of the large boot_strap_code
// array field.
#[cfg(feature = "bytemuck")]
#[allow(unsafe_code)]
unsafe impl Pod for MasterBootRecord {}
#[cfg(feature = "bytemuck")]
#[allow(unsafe_code)]
unsafe impl Zeroable for MasterBootRecord {}

impl MasterBootRecord {
    /// Return whether the [`boot_strap_code`] field is all zeros or not.
    ///
    /// [`boot_strap_code`]: Self::boot_strap_code
    #[must_use]
    pub fn is_boot_strap_code_zero(&self) -> bool {
        self.boot_strap_code.iter().all(|b| *b == 0)
    }

    /// Create a protective MBR for the given disk size.
    ///
    /// See section 5.2.3 "Protective MBR" of the UEFI Specification.
    #[must_use]
    pub fn protective_mbr(num_blocks: u64) -> Self {
        let size_in_lba = u32::try_from(num_blocks).unwrap_or(0xffff_ffff);

        Self {
            boot_strap_code: [0; 440],
            unique_mbr_disk_signature: [0; 4],
            unknown: [0; 2],
            partitions: [
                MbrPartitionRecord {
                    boot_indicator: 0,
                    // CHS=0,0,2
                    start_chs: Chs([0, 2, 0]),
                    os_indicator: 0xee,
                    end_chs: Chs::from_lba(
                        Lba(num_blocks - 1),
                        DiskGeometry::UNKNOWN,
                    )
                    .unwrap_or(Chs([0xff, 0xff, 0xff])),
                    starting_lba: U32Le::from_u32(1),
                    size_in_lba: U32Le::from_u32(size_in_lba - 1),
                },
                MbrPartitionRecord::default(),
                MbrPartitionRecord::default(),
                MbrPartitionRecord::default(),
            ],
            signature: [0x55, 0xaa],
        }
    }
}

impl Display for MasterBootRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("MasterBootRecord { boot_strap_code: ")?;

        if self.is_boot_strap_code_zero() {
            write!(f, "[0; {}]", self.boot_strap_code.len())?;
        } else {
            f.write_str("<non-zero>")?;
        }

        f.write_str(", unique_mbr_disk_signature: 0x")?;
        format_u8_slice_lower_hex_le(f, &self.unique_mbr_disk_signature)?;

        f.write_str(", unknown: ")?;
        format_u8_slice_lower_hex_le(f, &self.unknown)?;

        f.write_str(", partitions: [")?;
        for (i, partition) in self.partitions.iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            partition.fmt(f)?;
        }

        f.write_str("], signature: 0x")?;
        format_u8_slice_lower_hex_le(f, &self.signature)?;

        f.write_str(" }")
    }
}
