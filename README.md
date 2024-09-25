# pjdfstest.rs - Rust rewrite

pjdfstest is a file system test suite to assess the correctness of file system implementations in terms of POSIX compliance.
The test suite has been rewritten in Rust as part of the Google Summer of Code 2022 program.

## Build

### Requirements

The following dependencies are required to build the project:

#### Common

- Rust (1.72.1 or later)

#### Linux

- libacl1-dev (Debian/Ubuntu), libacl-devel (Fedora), acl (Arch)
- acl

```bash
cd rust
cargo run
```

## Documentation

The documentation is available at <https://saidsay-so.github.io/pjdfstest/>.

## Configuration file

A TOML configuration file can be used to specify parameters for the test suite.
A default configuration file is provided in the repository, which can be used as a template.

```toml
# Configuration for the pjdfstest runner

# This section allows enabling file system specific features.
# Please see the book for more details.
# A list of these features is provided when executing the runner with `-l`.
[features]
# File flags can be specified for OS which supports them.
# file_flags = ["UF_IMMUTABLE"]

# Here is an example with the `posix_fallocate` syscall.
posix_fallocate = {}

# Might use the key notation as well.
# [features.posix_fallocate]

[settings]
# naptime is the duration of various short sleeps.  It should be greater than
# the timestamp granularity of the file system under test.
naptime = 0.001
# Allow to run the EROFS tests, which require to remount the file system on which
# pjdsfstest is run as read-only.
allow_remount = false

# This section allows to modify the mechanism for switching users, which is required by some tests.
# [dummy_auth]
# An entry is the name of a user and its associated group.
# For now, the array requires exactly 3 entries.
# Please see the book for more details.
# entries = [
#   ["nobody", "nobody"],
#   nogroup instead for some Linux distros
#   ["nobody", "nogroup"],
#   ["tests", "tests"],
#   ["pjdfstest", "pjdfstest"],
# ]
```

Please refer to the [documentation](https://saidsay-so.github.io/pjdfstest/configuration-file.html) for more details.

## Command-line interface

_`pjdfstest [OPTIONS] [--] TEST_PATTERNS`_

- `-h, --help` - Print help message
- `-c, --configuration-file CONFIGURATION-FILE` - Path of the configuration file
- `-l, --list-features` - List opt-in features
- `-e, --exact` - Match names exactly
- `-v, --verbose` - Verbose mode
- `-p, --path PATH` - Path where the test suite will be executed
- `[--] TEST_PATTERNS` - Filter tests which match against the provided patterns

Example: `pjdfstest -c pjdfstest.toml chmod`

## Filter tests

It is possible to filter which tests should be ran, by specifying which parts should match.
Tests are usually identified by syscall and optionally the file type on which it operates.

## Rootless running

The test suite can be ran without privileges.
However, not all tests can be completed without privileges,
therefore the coverage will be incomplete.
For example, tests which need to switch users will not be run.

## Dummy users/groups

The test suite needs dummy users and groups to be set up.
This should be handled automatically when installing it via a package,
but they need to be created otherwise.
By default, the users (with the same name for the group associated to each of them) to create are:

- nobody
- tests
- pjdfstest

It is also possible to specify other users with the configuration file.

### Create users

#### FreeBSD

```bash
cat <<EOF | adduser -w none -S -f -
pjdfstest::::::Dummy User for pjdfstest:/nonexistent:/sbin/nologin:
EOF
```

#### Linux

```bash
cat <<EOF | newusers
tests:x:::Dummy User for pjdfstest:/:/usr/bin/nologin
pjdfstest:x:::Dummy User for pjdfstest:/:/usr/bin/nologin
EOF
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on the process for submitting pull requests to us
and the process for adding new tests.
Also, read the [documentation](https://saidsay-so.github.io/pjdfstest/test-declaration.html) on how to declare tests.
