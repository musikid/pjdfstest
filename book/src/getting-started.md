# Getting started

The test suite is as file system agnostic as possible. 
Its behavior can be modified with a configuration file.

## Command-line interface

*`pjdfstest [OPTIONS] [--] TEST_PATTERNS`*

* `-h, --help` - Print help message
* `-c, --configuration-file CONFIGURATION-FILE` - Path of the configuration file
* `-l, --list-features` - List opt-in features
* `-e, --exact` - Match names exactly
* `-v, --verbose` - Verbose mode
* `-p, --path PATH` - Path where the test suite will be executed
* `[--] TEST_PATTERNS` - Filter tests which match against the provided patterns

Example: `pjdfstest -c pjdfstest.toml chmod`