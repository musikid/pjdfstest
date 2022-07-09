# Test declaration

Test cases have the same anatomy than usual Rust tests, 
that is `unwrap`ing `Result`s and using assertion macros (`assert` and `assert_eq`),
the exception being that it should take a `&mut TestContext` parameter.
It might also take a `FileType` argument if required.
It also needs an additional declaration with the `test_case!` macro alongside the function, 
with the function name being the only mandatory argument.

For example:

```rust,ignore
// chmod/00.t:L58
crate::test_case! {ctime => [FileType::Regular, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn ctime(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    let ctime_before = stat(&path).unwrap().st_ctime;

    sleep(Duration::from_secs(1));

    chmod(&path, Mode::from_bits_truncate(0o111)).unwrap();

    let ctime_after = stat(&path).unwrap().st_ctime;
    assert!(ctime_after > ctime_before);
}
```

## Parameterization

### File types

Some test cases need to test over different file types.
The file types should be added at the end of the test case declaration,
with brackets and an fat arrow before (`=> [FileType::Regular]`).
The test function should also accept a `FileType` parameter to operate on.

For example:

```rust,ignore
crate::test_case! {change_perm => [FileType::Regular, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn change_perm(ctx: &mut TestContext, f_type: FileType) {
```

### Root privileges

Some tests may need root privileges to run.
To declare that a test function require root privileges, 
`root` should be added to its declaration.
For example:

```rust,ignore
crate::test_case!{change_perm, root}
```

The root requirement is automatically added for privileged file types,
namely block and char.

## Platform-specific functions 

Some functions (like `lchmod`) are not supported on every operating system.
When a test make use of such function, it is possible to restrain its compilation
to the supported operating systems, with the attribute `#[cfg(target_os = ...)]`.
It is also possible to apply this attribute on an aspect, or even on a syscall module.
For example:

```rust,ignore
#[cfg(target_os = "freebsd")]
mod lchmod;
```