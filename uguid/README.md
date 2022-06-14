# `uguid`

[![Crates.io](https://img.shields.io/crates/v/uguid)](https://crates.io/crates/uguid) 
[![Docs.rs](https://docs.rs/uguid/badge.svg)](https://docs.rs/uguid)

`no_std` library providing a GUID (Globally Unique Identifer) type, as
used in GPT disks, UEFI, and Microsoft Windows.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[`gpt_disk_io`]: https://crates.io/crates/gpt_disk_io

## Features

* `std`: Provides `std::error::Error` implementation for the error
  type. Off by default.

## License

Apache 2.0; see [`LICENSE`] for details.

[`LICENSE`]: https://github.com/google/gpt-disk-rs/blob/HEAD/LICENSE

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
