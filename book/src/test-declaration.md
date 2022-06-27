# Test declaration

A test function take a `&mut TestContext` parameter and returns a `TestResult`.

```rust,ignore
// chmod/00.t:L58
fn test_ctime(ctx: &mut TestContext) -> TestResult {
  for f_type in FileType::iter().filter(|&ft| ft == FileType::Symlink) {
      let path = ctx.create(f_type)?;
      let ctime_before = stat(&path)?.st_ctime;

      sleep(Duration::from_secs(1));

      chmod(&path, Mode::from_bits_truncate(0o111))?;

      let ctime_after = stat(&path)?.st_ctime;
      test_assert!(ctime_after > ctime_before);
  }

  Ok(())
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
pjdfs_test_case!(permission, { test: test_ctime, require_root: true });
```

## Platform-specific functions

Some functions (like `lchmod`) are not supported on every platform.
