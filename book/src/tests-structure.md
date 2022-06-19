# Structure

The tests should be grouped by syscalls, in the `tests/` folder.
Each folder then have a `mod.rs` file, 
which contains declarations of the modules inside this folder,
and a `pjdfs_group!` statement to export the test cases from these modules.
For example:

### Layout

```
chmod
├── lchmod
│   └── mod.rs
├── mod.rs
└── permission.rs
```

### mod.rs

```rust
mod permission;
mod lchmod;

crate::pjdfs_group!(chmod; permission::test_case, lchmod::test_case);
```

Each module inside a group should export a test case (with `pjdfs_test_case`),
which contains a list of test functions.
In our example, `chmod/permission.rs` would be:

```rust
use crate::{
    pjdfs_test_case,
    test::{TestContext, TestResult},
};

// chmod/00.t:L58
fn test_ctime(ctx: &mut TestContext) -> TestResult {
  for f_type in FileType::iter().filter(|&ft| ft == FileType::Symlink) {
      let path = ctx.create(f_type).map_err(TestError::CreateFile)?;
      let ctime_before = stat(&path)?.st_ctime;

      sleep(Duration::from_secs(1));

      chmod(&path, Mode::from_bits_truncate(0o111))?;

      let ctime_after = stat(&path)?.st_ctime;
      test_assert!(ctime_after > ctime_before);
  }

  Ok(())
}

pjdfs_test_case!(permission, { test: test_ctime });
```