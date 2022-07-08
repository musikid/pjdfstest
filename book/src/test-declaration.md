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

It is possible to give additional parameters to the test case macro,
to modify the execution of the tests or add requirements.

### File-system exclusive features

Some features are not available for every file system.
For tests requiring such features, the execution becomes opt-in.
They can be declared by adding them after eventual `root` requirement
and before the file types.
Every variant of `FileSystemFeature` can be specified.

```rust,ignore
#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, FileSystemFeature::Chflags, FileSystemFeature::FileFlags(&[FileFlags::SF_IMMUTABLE])}
#[cfg(target_os = "freebsd")]
fn eperm_immutable_flag(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Regular).unwrap();
    //TODO: Complete
}
```

#### File flags (MIGHT CHANGE)

It is possible to specify individual file flags for the tests which
requires it. `FileSystemFeature::FileFlags` takes a slice parameter,
which is made of the used file flags.

##### Warning: There is also a `FileFlags` defined for `nix`.

```rust,ignore
test_case! { ..., FileSystemFeature::FileFlags(&[FileFlags::UF_IMMUTABLE, FileFlags::SF_IMMUTABLE])} }
```

##### NOTE: The file flags feature is the only one to have a parameter, and probably should stay that way.

#### Adding features



### File types

Some test cases need to test over different file types.
The file types should be added at the end of the test case declaration,
within brackets, with a fat arrow before (`=> [FileType::Regular]`).
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

## TODO: Platform-specific functions 

Some functions (like `lchmod`) are not supported on every platform.
