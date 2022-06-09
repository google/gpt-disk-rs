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

#[cfg(feature = "std")]
use {
    anyhow::Result,
    gpt_disk_io::gpt_disk_types::BlockSize,
    gpt_disk_io::{Disk, StdBlockIo},
    std::{env, fs},
};

// To create a disk to test this you can use truncate and sgdisk. For example:
//
// truncate --size 10MiB disk.bin
// sgdisk disk.bin --new=1:2048:4096 --change-name=1:'hello world!' --print
// cargo run --features=std --example reader disk.bin

#[cfg(feature = "std")]
fn main() -> Result<()> {
    let disk_path = env::args().nth(1).expect("one argument is required");
    println!("opening {} for reading", disk_path);

    let mut file = fs::File::open(disk_path)?;

    let mut block_buf = vec![0u8; 512];

    let block_io = StdBlockIo::new(&mut file, BlockSize::BS_512);
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
