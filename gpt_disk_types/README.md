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

No features are enabled by default.

* `bytemuck`: Implements bytemuck's `Pod` and `Zeroable` traits for many
  of the types in this crate. Also enables some methods that rely on
  byte access.
* `std`: Provides `std::error::Error` implementations for all of the
  error types.
  
## Minimum Supported Rust Version (MSRV)

The current MSRV is 1.60.0 due to the use of the Cargo [`dep:`]
feature. Feel free to file an issue or create a PR if you have a use
case that requires an older version.

[`dep:`]: https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html#new-syntax-for-cargo-features

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
