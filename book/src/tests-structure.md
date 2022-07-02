# Structure

The tests should be grouped by syscalls, in the `tests/` folder.
Each folder then have a `mod.rs` file, 
which contains declarations of the modules inside this folder,
and a `group!` statement to export the test cases from these modules.
For example:

### Layout

```
chmod (syscall/test group)
├── errno.rs (test case)
├── mod.rs (test group declaration)
└── permission.rs (test case)
```

### mod.rs

```rust,ignore
mod permission;
mod lchmod;
```

Each module inside a group should register its test cases with `test_case!`.
In our example, `chmod/permission.rs` would be:

```rust,ignore
test_case!{ctime, root, Syscall::Chmod}
use crate::{
    test_case,
    test::{TestContext, TestResult},
};

// chmod/00.t:L58
fn ctime(ctx: &mut TestContext) -> TestResult {
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
```
