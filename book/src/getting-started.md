# Getting started

The test suite is as file system agnostic as possible
and tries to comply with the POSIX specification.
Typically, tests which make use of non-POSIX features
are opt-in and only tests for syscalls which must be available on every POSIX system are ran.
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

It is possible to filter which tests should be run by specifying which parts should match.
Tests are usually identified by syscall and optionally the file type on which it operates.

## Rootless running

The test suite can be run without privileges.
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
