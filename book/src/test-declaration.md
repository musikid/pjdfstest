# Test declaration

A test function should take a `&mut TestContext` parameter.
It has the same anatomy than usual Rust tests, that is `unwrap`ing `Result`s and using assertion macros (`assert` and `assert_eq`).
For example:

```rust,ignore
// chmod/00.t:L58
crate::test_case!{Syscall::Chmod, ctime}
fn ctime(ctx: &mut TestContext) {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type).unwrap();
        let ctime_before = stat(&path).unwrap().st_ctime;

        sleep(Duration::from_secs(1));

        chmod(&path, Mode::from_bits_truncate(0o111)).unwrap();

        let ctime_after = stat(&path).unwrap().st_ctime;
        assert!(ctime_after > ctime_before);
    }
}
```

## Parameterization

### File types

Some tests need to test different file types.
For now, a for loop which iterates on the types is used, but it should change in the future for a
better structure (especially because of tests with `sleep`, which cannot be easily parallelized with a loop, see [#1](https://github.com/musikid/pjdfstest/issues/1)).

```rust,ignore
for f_type in FileType::iter() {
}
```

Since it is an iterator, usual functions like `filter` works.

```rust,ignore
for f_type in FileType::iter().filter(|&ft| ft == FileType::Symlink) {
}
```

### Root privileges

Some tests may need root privileges to run.
Especially, all the tests which involves creating a block/char file need those.

To declare that a test function require root privileges, 
`require_root: true` should be added to its declaration.

For example:

```rust
test_case!{change_perm, root, Syscall::Chmod}
```

## TODO: Platform-specific functions 

Some functions (like `lchmod`) are not supported on every platform.
