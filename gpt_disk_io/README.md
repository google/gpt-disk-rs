# `gpt_disk_io`

`no_std` library for reading and writing [GPT] (GUID Partition Table)
disk data structures through a block IO interface.

[GPT]: https://en.wikipedia.org/wiki/GUID_Partition_Table

## Features

* `std`: Provides `std::error::Error` implementations for all of the
  error types. Off by default.
