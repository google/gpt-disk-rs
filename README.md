# gpt-disk-rs

`no_std` libraries related to [GPT] (GUID Partition Table) disk data.

There are two Rust packages in this repository, `gpt_disk_types` and
`gpt_disk_io`. The `gpt_disk_types` package provides flexible types for
all the GPT types defined in the [UEFI Specification]. The types are
designed to ensure correct endianness regardless of host platform, and
can be used even with corrupted input data. The `gpt_disk_io` package
adds types for reading and writing GPT data to an abstract disk
interface. This interface can be implemented for any backend that
supports block-level IO.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[UEFI Specification]: https://uefi.org/specifications

## Code layout

### `gpt_disk_types`

* `block.rs`: Numeric types for addressing blocks.
* `crc32.rs`: CRC32 type.
* `guid.rs`: Globally-unique identifier type.
* `header.rs`: GPT header and related types.
* `mbr.rs`: Legacy master boot record types.
* `num.rs`: Little-endian integer types.
* `partition_array.rs`: GPT partition array types.
* `partition_entry.rs`: GPT partition array entry types.
* `std_support.rs`: Provides `std` trait impls when the `std` feature is enabled.

### `gpt_disk_io`

* `block_io.rs`: BlockIo trait for generic read/write operations.
* `slice_block_io.rs`: In-memory byte slice implementations of BlockIo.
* `disk.rs`: Read and write GPT data from a block device.
* `std_support.rs`: Provides `std` trait impls when the `std` feature is enabled.

## License

Apache 2.0; see [`LICENSE`](LICENSE) for details.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
