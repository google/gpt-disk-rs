# `gpt_disk_types`

[![Crates.io](https://img.shields.io/crates/v/gpt_disk_types)](https://crates.io/crates/gpt_disk_types) 
[![Docs.rs](https://docs.rs/gpt_disk_types/badge.svg)](https://docs.rs/gpt_disk_types)

`no_std` library providing [GPT] (GUID Partition Table) data
structures. The types are designed to ensure correct endianness
regardless of host platform, and can be used even with corrupted input
data.

See also the [`gpt_disk_io`] package.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[`gpt_disk_io`]: https://crates.io/crates/gpt_disk_io

## Features

* `std`: Provides `std::error::Error` implementations for all of the
  error types. Off by default.

## License

Apache 2.0; see [`LICENSE`] for details.

[`LICENSE`]: https://github.com/google/gpt-disk-rs/blob/HEAD/LICENSE

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
