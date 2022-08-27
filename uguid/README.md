# `uguid`

[![Crates.io](https://img.shields.io/crates/v/uguid)](https://crates.io/crates/uguid) 
[![Docs.rs](https://docs.rs/uguid/badge.svg)](https://docs.rs/uguid)

`no_std` library providing a GUID (Globally Unique Identifer) type, as
used in GPT disks, UEFI, and Microsoft Windows.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[`gpt_disk_io`]: https://crates.io/crates/gpt_disk_io

## Features

No features are enabled by default.

* `bytemuck`: Implements bytemuck's `Pod` and `Zeroable` traits for `Guid`.
* `serde`: Implements serde's `Serialize` and `Deserialize` traits for `Guid`.
* `std`: Provides `std::error::Error` implementation for the error type.

## Minimum Supported Rust Version (MSRV)

The current MSRV is 1.60.0 due to the use of the Cargo [`dep:`]
feature. Feel free to file an issue or create a PR if you have a use
case that requires an older version.

[`dep:`]: https://blog.rust-lang.org/2022/04/07/Rust-1.60.0.html#new-syntax-for-cargo-features

## License

Licensed under either of [Apache License, Version 2.0] or [MIT license]
at your option.

[Apache License, Version 2.0]: https://github.com/google/gpt-disk-rs/blob/HEAD/LICENSE-APACHE
[MIT license]: https://github.com/google/gpt-disk-rs/blob/HEAD/LICENSE-MIT

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
