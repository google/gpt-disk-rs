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

use common::check_derives;
use gpt_disk_types::{
    BlockSize, GptPartitionEntryArrayLayout, GptPartitionEntrySize, Lba,
};

#[test]
fn test_partition_entry_array_layout() {
    check_derives::<GptPartitionEntryArrayLayout>();

    let layout = GptPartitionEntryArrayLayout {
        start_lba: Lba(2),
        entry_size: GptPartitionEntrySize::new(256).unwrap(),
        num_entries: 128,
    };
    assert_eq!(layout.num_blocks(BlockSize::BS_512).unwrap(), 64);
    assert_eq!(
        layout
            .num_bytes_rounded_to_block(BlockSize::BS_512)
            .unwrap(),
        64 * 512
    );
    assert_eq!(layout.num_bytes_exact().unwrap(), 256 * 128);

    let bs767 = BlockSize::new(512 + 256 - 1).unwrap();
    assert_eq!(layout.num_blocks(bs767).unwrap(), 43);
    assert_eq!(layout.num_bytes_rounded_to_block(bs767).unwrap(), 43 * 767);
    assert_eq!(layout.num_bytes_exact().unwrap(), 256 * 128);

    assert_eq!(layout.num_blocks_as_usize(bs767).unwrap(), 43);
    assert_eq!(
        layout.num_bytes_rounded_to_block_as_usize(bs767).unwrap(),
        43 * 767
    );
    assert_eq!(layout.num_bytes_exact_as_usize().unwrap(), 256 * 128);
}
