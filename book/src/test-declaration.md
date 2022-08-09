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
crate::test_case! {ctime => [Regular, Fifo, Block, Char, Socket]}
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
When a test needs such feature,
a variant of `FileSystemFeature` corresponding to this feature should be specified
after potential `root` requirement and before file flags.
Multiple features can be specified, each separated by a comma `,` separator.

For example:

```rust,ignore
#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, FileSystemFeature::Chflags, FileSystemFeature::PosixFallocate ...}
```

#### Adding features

New features can be added to the `FileSystemFeature` enum.
A description of the feature should be provided as documentation.

#### File flags

**NOTE: This feature is not supported by all POSIX systems, 
therefore its use needs a `#[cfg(target_os = ...)]` attribute specifying the relevant system(s).**

It is possible to specify individual file flags for the tests which
require it. They can be specified by appending `FileFlags` variants after a `;` separator,
after potential `root` requirement and features.

```rust,ignore
#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, root, FileSystemFeature::Chflags; FileFlags::SF_IMMUTABLE, FileFlags::UF_IMMUTABLE}
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

### File types

Some test cases need to test over different file types.
The file types should be added at the end of the test case declaration,
within brackets and with a fat arrow before (`=> [Regular]`).
The test function should also accept a `FileType` parameter to operate on.

For example:

```rust,ignore
crate::test_case! {change_perm, root, FileSystemFeature::Chflags; FileFlags::SF_IMMUTABLE, FileFlags::UF_IMMUTABLE 
=> [Regular, Fifo, Block, Char, Socket]}
fn change_perm(ctx: &mut TestContext, f_type: FileType) {
```

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

## Serialized test cases

Some test cases need functions only available when they are run serialized, especially when they affect the whole process.
An example is changing user (`SerializedTestContext::as_user`).
To have access to these functions, the test should be declared with a `SerializedTestContext`
parameter in place of `TestContext` 
and the `serialized` keyword should be prepended before features.

For example:

```rust,ignore
crate::test_case! {
    /// The file mode of a newly created file should not affect whether
    /// posix_fallocate will work, only the create args
    /// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=154873
    affected_only_create_flags, serialized, root, FileSystemFeature::PosixFallocate
}
fn affected_only_create_flags(ctx: &mut SerializedTestContext) {
    ctx.as_user(Some(Uid::from_raw(65534)), None, || {
        let path = subdir.join("test1");
        let file = open(&path, OFlag::O_CREAT | OFlag::O_RDWR, Mode::empty()).unwrap();
        assert!(posix_fallocate(file, 0, 1).is_ok());
    });
} 
```
