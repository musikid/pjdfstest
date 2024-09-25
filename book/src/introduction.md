# Introduction

PJDFSTest is a file system test suite focused on POSIX compliance,
primarily for FreeBSD file systems.
It was originally written to validate the ZFS port to FreeBSD,
but it now supports multiple operating systems and file systems.

This is a complete rewrite of the original test suite in Rust,
as part of the Google Summer of Code 2022 program
(<https://summerofcode.withgoogle.com/archive/2022/projects/6XPYWLzJ>).

**NOTE: The documentation is still a work-in-progress**

## Build

```bash
cd rust
cargo run
```

### Run as root

```bash
cd rust
cargo build && sudo ./target/debug/pjdfstest
```

## Contributing

Please read the [CONTRIBUTING.md](CONTRIBUTING.md) file on how to contribute to this project.
In addition to this book, you can also find the crate documentation by running `cargo doc --open`
in the `rust` directory or by visiting the [documentation](/doc/pjdfstest).
