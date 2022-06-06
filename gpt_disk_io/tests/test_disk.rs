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

mod common;

use anyhow::Result;
use common::{
    create_partition_entry, create_primary_header, create_secondary_header,
};
use gpt_disk_io::{BlockIo, Disk, DiskError, MutSliceBlockIo};
use gpt_disk_types::{BlockSize, GptPartitionEntryArray};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;
#[cfg(feature = "std")]
use {
    gpt_disk_io::StdBlockIo,
    std::fs::{File, OpenOptions},
};

fn test_disk_read<Io>(block_io: Io) -> Result<(), DiskError<Io::Error>>
where
    Io: BlockIo,
{
    let mut block_buf = vec![0u8; 512];
    let mut disk = Disk::new(block_io)?;

    let primary_header = disk.read_primary_gpt_header(&mut block_buf)?;
    assert_eq!(primary_header, create_primary_header());

    let secondary_header = disk.read_secondary_gpt_header(&mut block_buf)?;
    assert_eq!(secondary_header, create_secondary_header());

    let expected_partition_entry = create_partition_entry();

    // Check the primary partition entry array.
    let primary_partition_entry = disk
        .gpt_partition_entry_array_iter(
            primary_header.get_partition_entry_array_layout().unwrap(),
            &mut block_buf,
        )?
        .next()
        .unwrap()?;
    assert_eq!(primary_partition_entry, expected_partition_entry);

    // Check the secondary partition entry array.
    let second_partition_entry = disk
        .gpt_partition_entry_array_iter(
            primary_header.get_partition_entry_array_layout().unwrap(),
            &mut block_buf,
        )?
        .next()
        .unwrap()?;
    assert_eq!(second_partition_entry, expected_partition_entry);

    Ok(())
}

fn test_disk_write<Io>(block_io: Io) -> Result<(), DiskError<Io::Error>>
where
    Io: BlockIo,
{
    let bs = BlockSize::B512;
    let mut block_buf = vec![0u8; bs.to_usize().unwrap()];
    let mut disk = Disk::new(block_io)?;

    let primary_header = create_primary_header();
    let secondary_header = create_secondary_header();
    let partition_entry = create_partition_entry();

    disk.write_protective_mbr(&mut block_buf)?;
    disk.write_primary_gpt_header(&primary_header, &mut block_buf)?;
    disk.write_secondary_gpt_header(&secondary_header, &mut block_buf)?;

    let layout = primary_header.get_partition_entry_array_layout().unwrap();
    let mut bytes =
        vec![0; layout.num_bytes_rounded_to_block_as_usize(bs).unwrap()];
    let mut entry_array =
        GptPartitionEntryArray::new(layout, bs, &mut bytes).unwrap();
    *entry_array.get_partition_entry_mut(0).unwrap() = partition_entry;
    disk.write_gpt_partition_entry_array(&entry_array)?;

    entry_array.set_start_lba(secondary_header.partition_entry_lba.into());
    disk.write_gpt_partition_entry_array(&entry_array)?;

    disk.flush()?;

    Ok(())
}

fn run_cmd(cmd: &mut Command) -> Result<()> {
    let o = cmd.output()?;
    assert!(o.status.success());
    Ok(())
}

fn create_empty_file(path: &Path, size: &str) -> Result<()> {
    run_cmd(Command::new("truncate").args(&["--size", size]).arg(path))
}

fn test_with_mut_slice(sgdisk_path: &Path) -> Result<()> {
    // Test read.
    let mut contents = fs::read(&sgdisk_path)?;
    test_disk_read(MutSliceBlockIo::new(&mut contents, BlockSize::B512))
        .unwrap();

    // Test write.
    let mut new_contents = vec![0; contents.len()];
    test_disk_write(MutSliceBlockIo::new(&mut new_contents, BlockSize::B512))
        .unwrap();
    assert_eq!(contents, new_contents);

    Ok(())
}

#[cfg(feature = "std")]
fn test_with_file(tmp_path: &Path, sgdisk_path: &Path) -> Result<()> {
    // Test read.
    let mut file = File::open(&sgdisk_path)?;
    test_disk_read(StdBlockIo::new(&mut file, BlockSize::B512))?;

    // Test write.
    let new_disk_path = tmp_path.join("new_disk.bin");
    create_empty_file(&new_disk_path, "4MiB")?;
    let mut new_file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .open(&new_disk_path)?;
    test_disk_write(StdBlockIo::new(&mut new_file, BlockSize::B512)).unwrap();
    let expected_bytes = fs::read(&sgdisk_path)?;
    let actual_bytes = fs::read(&new_disk_path)?;
    assert_eq!(expected_bytes, actual_bytes);

    Ok(())
}

#[test]
#[cfg_attr(miri, ignore)]
fn test_disk() -> Result<()> {
    if Command::new("sgdisk").arg("--version").status().is_err() {
        panic!("failed to run sgdisk, is it installed?");
    }

    let tmp_dir = TempDir::new()?;
    let tmp_path = tmp_dir.path();

    let sgdisk_path = tmp_path.join("disk.bin");
    create_empty_file(&sgdisk_path, "4MiB")?;
    run_cmd(Command::new("sgdisk").arg(&sgdisk_path).args(&[
        "--disk-guid=57a7feb6-8cd5-4922-b7bd-c78b0914e870",
        "--new=1:2048:4096",
        "--change-name=1:hello world!",
        "--partition-guid=1:37c75ffd-8932-467a-9c56-8cf1f0456b12",
        "--typecode=1:ccf0994f-f7e0-4e26-a011-843e38aa2eac",
    ]))?;

    test_with_mut_slice(&sgdisk_path)?;

    #[cfg(feature = "std")]
    test_with_file(&tmp_path, &sgdisk_path)?;

    Ok(())
}
