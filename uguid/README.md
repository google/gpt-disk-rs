# `uguid`

[![Crates.io](https://img.shields.io/crates/v/uguid)](https://crates.io/crates/uguid) 
[![Docs.rs](https://docs.rs/uguid/badge.svg)](https://docs.rs/uguid)

`no_std` library providing a GUID (Globally Unique Identifier) type, as
used in GPT disks, UEFI, and Microsoft Windows.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table

## Features

No features are enabled by default.

* `bytemuck`: Implements bytemuck's `Pod` and `Zeroable` traits for `Guid`.
* `serde`: Implements serde's `Serialize` and `Deserialize` traits for `Guid`.
* `std`: Currently has no effect.

## Minimum Supported Rust Version (MSRV)

The current MSRV is 1.81.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE)
or [MIT license](LICENSE-MIT) at your option.

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
