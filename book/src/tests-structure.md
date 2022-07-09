# Structure

The tests should be grouped by syscalls, in the `tests/` folder.
Each folder then have a `mod.rs` file, 
which contains declarations of the modules inside this folder.
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
mod errno;
```

Each module inside a group should register its test cases with `test_case!`.
In our example, `chmod/permission.rs` would be:

```rust,ignore
// chmod/00.t:L58
crate::test_case! {ctime => [FileType::Regular, FileType::Dir, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn ctime(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    let ctime_before = stat(&path).unwrap().st_ctime;

    sleep(Duration::from_secs(1));

    chmod(&path, Mode::from_bits_truncate(0o111)).unwrap();

    let ctime_after = stat(&path).unwrap().st_ctime;
    assert!(ctime_after > ctime_before);
}
```
