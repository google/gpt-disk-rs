# `gpt_disk_io`

[![Crates.io](https://img.shields.io/crates/v/gpt_disk_io)](https://crates.io/crates/gpt_disk_io) 
[![Docs.rs](https://docs.rs/gpt_disk_io/badge.svg)](https://docs.rs/gpt_disk_io)

`no_std` library for reading and writing [GPT] (GUID Partition Table)
disk data structures through a block IO interface.

See also the [`gpt_disk_types`] package.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[`gpt_disk_types`]: https://crates.io/crates/gpt_disk_types

## Features

No features are enabled by default.

* `alloc`: Enables `Vec` implementation of `BlockIoAdapter`.
* `std`: Enables `std::io` implementations of `BlockIoAdapter`.
  
## Minimum Supported Rust Version (MSRV)

The current MSRV is 1.81.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
