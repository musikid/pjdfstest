# Introduction

PJDFSTest is a file system test suite.
It was originally written to validate the ZFS port to FreeBSD,
but it now supports multiple operating systems and file systems.
This is a complete rewrite of the original test suite in Rust.

### NOTE: The documentation is still a work-in-progress.

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

