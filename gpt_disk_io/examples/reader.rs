// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "std")]
use {
    gpt_disk_io::gpt_disk_types::BlockSize,
    gpt_disk_io::{BlockIoAdapter, Disk},
    std::{env, error, fs},
};

// To create a disk to test this you can use truncate and sgdisk. For example:
//
// truncate --size 10MiB disk.bin
// sgdisk disk.bin --new=1:2048:4096 --change-name=1:'hello world!' --print
// cargo run --features=std --example reader disk.bin

#[cfg(feature = "std")]
fn main() -> Result<(), Box<dyn error::Error>> {
    let disk_path = env::args().nth(1).expect("one argument is required");
    println!("opening {} for reading", disk_path);

    let file = fs::File::open(disk_path)?;

    let mut block_buf = vec![0u8; 512];

    let block_io = BlockIoAdapter::new(file, BlockSize::BS_512);
    let mut disk = Disk::new(block_io)?;

    let primary_header = disk.read_primary_gpt_header(&mut block_buf)?;
    println!("{}", primary_header);
    assert!(primary_header.is_signature_valid());

    let layout = primary_header.get_partition_entry_array_layout()?;
    for entry in disk.gpt_partition_entry_array_iter(layout, &mut block_buf)? {
        let entry = entry?;
        if entry.is_used() {
            println!("{}", entry);
        }
    }

    Ok(())
}

#[cfg(not(feature = "std"))]
fn main() {
    panic!("this program must be compiled with the 'std' feature");
}
