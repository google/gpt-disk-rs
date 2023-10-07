// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// TODO
#![allow(missing_docs)]

use crate::{BlockIo, BlockIoAdapter, Disk};
use gpt_disk_types::{
    BlockSize, GptHeader, GptPartitionAttributes, GptPartitionType, Guid, Lba,
    LbaLe, U32Le,
};
use std::fs::File;
use std::ops::RangeInclusive;
use std::path::Path;

#[derive(Debug)]
pub enum GptError {
    // TODO
    Io,
    Random(getrandom::Error),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Partition {
    pub partition_type: GptPartitionType,
    pub guid: Guid,
    pub lba_range: RangeInclusive<u64>,
    pub attributes: GptPartitionAttributes,
    pub name: String,
}

impl Partition {
    pub fn write_data<Io: BlockIo>(
        &self,
        disk: &mut Disk<Io>,
        data: &[u8],
    ) -> Result<(), GptError> {
        // TODO: check that data fits in the lba range
        // TODO: fix unwrap
        let mut block_buf = vec![0; disk.block_size().to_usize().unwrap()];
        disk.write_data(Lba(*self.lba_range.start()), data, &mut block_buf)
            .map_err(|_| GptError::Io)
    }
}

impl Default for Partition {
    fn default() -> Self {
        Self {
            partition_type: GptPartitionType::UNUSED,
            // TODO: should this be random?
            guid: Guid::ZERO,
            lba_range: 0..=0,
            attributes: GptPartitionAttributes::default(),
            name: String::new(),
        }
    }
}

pub struct DiskBuilder {
    block_size: BlockSize,
    size_in_bytes: u64,
    num_partitions: u32,

    // if None: random
    guid: Option<Guid>,
}

impl DiskBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_block_size(&mut self, block_size: BlockSize) {
        self.block_size = block_size;
    }

    pub fn set_size_in_bytes(&mut self, size: u64) {
        self.size_in_bytes = size;
    }

    pub fn set_size_in_mebibyte(&mut self, size: u64) {
        self.size_in_bytes = size * (1024 * 1024);
    }

    pub fn set_num_partitions(&mut self, num: u32) {
        self.num_partitions = num;
    }

    pub fn set_guid(&mut self, guid: Guid) {
        self.guid = Some(guid);
    }

    pub fn create_disk_in_memory(
        &self,
    ) -> Result<Disk<BlockIoAdapter<Vec<u8>>>, GptError> {
        // TODO: fix unwraps
        let len = usize::try_from(
            self.block_size.round_u64_up(self.size_in_bytes).unwrap(),
        )
        .unwrap();
        let data = vec![0; len];
        let block_io = BlockIoAdapter::new(data, self.block_size);
        Ok(Disk::new(block_io).unwrap())
    }
}

impl Default for DiskBuilder {
    fn default() -> Self {
        // Get the smallest possible size of the disk given a 512-byte block size.
        //
        // + 1 block for the MBR
        // + 1 block for the GPT header
        // + 32 blocks for the partition entry array (to meet the
        //   required minimum of 16KiB)
        //
        // No space is reserved for actual partition data.
        //
        // Double that, to cover both the primary and secondary GPT.
        let min_disk_size_in_blocks = (1 + 1 + 32) * 2;

        Self {
            block_size: BlockSize::BS_512,
            // Set the default size to the smallest possible disk size
            // for this block size.
            size_in_bytes: min_disk_size_in_blocks * 512,
            num_partitions: 0,
            guid: None,
        }
    }
}

enum DiskImpl {}

pub struct Disk(DiskImpl);

#[derive(Default)]
pub struct Gpt {
    pub block_size: BlockSize,
    // TODO primary: GptHeader,
    pub partitions: Vec<Partition>,
    // pub num_blocks: usize,
    pub size_in_bytes: u64,
    // TODO pub align_in_bytes: u32
    // TODO: if None, random
    pub guid: Option<Guid>,
}

impl Gpt {
    pub fn create_disk_in_memory(
        &self,
    ) -> Result<Disk<BlockIoAdapter<Vec<u8>>>, GptError> {
        // TODO: fix unwraps
        let len = usize::try_from(
            self.block_size.round_u64_up(self.size_in_bytes).unwrap(),
        )
        .unwrap();
        let data = vec![0; len];
        let block_io = BlockIoAdapter::new(data, self.block_size);
        let mut disk = Disk::new(block_io).unwrap();
        let mut block_buf = vec![0; self.block_size.to_usize().unwrap()];
        disk.write_protective_mbr(&mut block_buf).unwrap();

        let mut primary_header = GptHeader {
            my_lba: LbaLe::from_u64(1),
            alternate_lba: LbaLe::from_u64(disk.num_blocks().unwrap() - 1),
            // TODO: in practice this value is usually correct, but if a
            // different block size is set, or if there are an absurd
            // number of partitions, it will need adjustment.
            first_usable_lba: LbaLe::from_u64(34),
            // TODO
            last_usable_lba: LbaLe::from_u64(
                disk.num_blocks().unwrap() - 1 - 32,
            ),
            disk_guid: self.get_disk_guid()?,
            partition_entry_lba: LbaLe::from_u64(2),
            // TODO
            number_of_partition_entries: U32Le::from_u32(128),
            ..Default::default()
        };
        // TODO: update partition_entry_array_crc32
        primary_header.update_header_crc32();
        disk.write_primary_gpt_header(&primary_header, &mut block_buf)
            .unwrap();

        // TODO
        let mut secondary_header = primary_header.clone();
        secondary_header.update_header_crc32();
        disk.write_secondary_gpt_header(&secondary_header, &mut block_buf)
            .unwrap();

        Ok(disk)
    }

    pub fn write_disk_to_path<P>(
        &self,
        _path: P,
    ) -> Result<Disk<BlockIoAdapter<File>>, GptError>
    where
        P: AsRef<Path>,
    {
        todo!()
    }

    fn get_disk_guid(&self) -> Result<Guid, GptError> {
        if let Some(guid) = self.guid {
            Ok(guid)
        } else {
            let mut random_bytes = [0; 16];
            getrandom::getrandom(&mut random_bytes)
                .map_err(GptError::Random)?;
            Ok(Guid::from_random_bytes(random_bytes))
        }
    }

    // fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, GptError> {
    //     todo!()
    // }

    // TODO
    // fn from_file() -> {}

    // fn from_bytes(bytes: &[u8]) -> Result<Self, GptError> {
    //     let block_io = SliceBlockIo::new(&mut disk_storage, bs);
    //     todo!()
    // }
}

// TODO: figuring out usage here, move these tests to tests dir.
#[cfg(test)]
mod tests {
    use super::*;
    // use gpt_disk_types::GptPartitionType;

    #[test]
    fn test_todo() -> Result<(), GptError> {
        let db = DiskBuilder::new();
        assert_eq!(db.block_size, BlockSize::BS_512);
        let _disk1 = db.create_disk_in_memory();
        // let disk2 = db.create_disk_in_file("some_file");
        // let disk3 = db.create_disk_in_file("/dev/sdx");

        Ok(())
    }

    //#[test]
    // fn test_todo() -> Result<(), GptError> {
    //     // Gpt is entirely in memory and doesn't know the size of the
    //     // disk. It contains the block size and the partition list. It
    //     // doesn't check anything until you write out the partitions.

    //     // Can create it empty (just need block size), or try to scan
    //     // from an existing disk.

    //     // What about partition access? Can return offsets or something
    //     // like that...

    //     let mut gpt = Gpt {
    //         // 4 MiB
    //         size_in_bytes: 4 * 1024 * 1024,
    //         ..Gpt::default()
    //     };
    //     // Implicit alignment default to 1MiB?
    //     gpt.partitions.push(Partition {
    //         partition_type: GptPartitionType::EFI_SYSTEM,
    //         lba_range: 2048..=4096,
    //         name: "ESP".parse().unwrap(),
    //         ..Default::default()
    //     });
    //     let mut disk = gpt.create_disk_in_memory()?;
    //     gpt.partitions[0]
    //         .write_data(&mut disk, &[0, 1, 2, 3])
    //         .unwrap();
    //     //disk_data.write_partition

    //     //gpt.write_partition(disk, 0, [0, 1, 2, 3]);

    //     let mut _disk = gpt.write_disk_to_path("some/path");

    //     // let mut gpt = Gpt::from_path("/dev/sda").unwrap();
    //     // Also from_file, from_disk, etc...

    //     Ok(())
    // }

    // #[test]
    // fn test_todo_old() {
    //     // Read GPT from existing disk, provide read+write access to
    //     // partition data.

    //     // Read GPT from existing disk, modify GPT, write to disk.

    //     let mut disk = Disk::create(
    //         "/some/new/file.bin",
    //         // 4 MiB
    //         4 * 1024 * 1024,
    //         BlockSize::BS_512,
    //     )?;
    //     Gpt::new(&mut disk)?;
    //     disk.write_gpt(gpt)?;

    //     Gpt::create_disk(
    //         "/some/new/file.bin",
    //         // 4 MiB
    //         4 * 1024 * 1024,
    //         BlockSize::BS_512,
    //     );

    //     // let mut gpt = Gpt::open_disk("/some/existing/file.bin")?;
    //     // gpt.read_blo

    //     // Create GPT from scratch, write to disk.
    //     let mut gpt = Gpt::create_disk(
    //         "/some/new/file.bin",
    //         // 4 MiB
    //         4 * 1024 * 1024,
    //         BlockSize::BS_512,
    //     );
    //     gpt.add_partition(Partition {
    //         partition_type: GptPartitionType::EFI_SYSTEM,
    //         lba_range: 2048..=4096,
    //         name: "ESP",
    //         ..Default::default()
    //     });
    //     gpt.write_to("/some/new/file.bin")?;

    //     // TODO gpt.write_to(some_disk)?;

    //     // Can all of these be combined into one type? Should they be?

    //     // Consider, for example, creating a GPT from scratch. If we do
    //     // that, and then also want to be able to read from the disk, we
    //     // need to have a backing disk available. Could be either at
    //     // creation time, or set it later...
    // }
}
