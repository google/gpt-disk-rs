# `gpt_disk_io`

[![Crates.io](https://img.shields.io/crates/v/gpt_disk_io)](https://crates.io/crates/gpt_disk_io) 
[![Docs.rs](https://docs.rs/gpt_disk_io/badge.svg)](https://docs.rs/gpt_disk_io)

`no_std` library for reading and writing [GPT] (GUID Partition Table)
disk data structures through a block IO interface.

See also the [`gpt_disk_types`] package.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table
[`gpt_disk_types`]: https://crates.io/crates/gpt_disk_types

## Features

* `std`: Enables the `StdBlockIo` type, as well as `std::error::Error`
  implementations for all of the error types. Off by default.

## License

Apache 2.0; see [`LICENSE`] for details.

[`LICENSE`]: https://github.com/google/gpt-disk-rs/blob/HEAD/LICENSE

## Disclaimer

This project is not an official Google project. It is not supported by
Google and Google specifically disclaims all warranties as to its quality,
merchantability, or fitness for a particular purpose.
