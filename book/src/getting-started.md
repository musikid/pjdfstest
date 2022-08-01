# Getting started

The test suite is as file system agnostic as possible
and tries to comply with the POSIX specification.

Its behavior can be modified with a configuration file.
By default, only tests for syscalls which must be available on every POSIX system are ran.
It can be configured with the configuration file, by specifying additional features
supported by the file system/operating system.

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

## Filter tests

It is possible to filter which tests should be ran, by specifying which parts should match.
Tests are usually identified by syscall and optionally the file type on which it operates.

## Rootless running

The test suite can be run without privileges.
However, not all tests can be completed without privileges,
therefore the coverage will be incomplete.
