# PJDFSTest.rs - Rust rewrite

## Architecture

The package is made of:

- a library, which contains all the tests,
- a binary, which is the test runner.

### Library

The library exports modules, which contains groups of test cases, generally grouped by syscall.
A test case is itself composed of multiple test functions. 

### Binary

The binary is the test runner.

## Writing tests

The tests should be grouped by syscalls, in the `tests/` folder.
Each folder then have a `mod.rs` file, 
which contains declarations of the modules inside this folder,
and a `pjdfs_group!` statement to export these tests.
For example:

#### Layout

```
chmod
├── lchmod
│   └── mod.rs
├── mod.rs
└── permission.rs
```

#### mod.rs

```rust
mod permission;
mod lchmod;

crate::pjdfs_group!(chmod; permission::test_case);
```


