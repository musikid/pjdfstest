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
When a test need such feature, a variant of `FileSystemFeature` corresponding to this feature should be specified,
by adding it after eventual `root` requirement and before the file types.
Multiple features can be specified, with a comma `,` separator.

For example:

```rust,ignore
#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, FileSystemFeature::Chflags, FileSystemFeature::PosixFallocate ...}
```

#### File flags

**NOTE: This feature is not supported by all POSIX systems, 
therefore its use needs a `#[cfg(target_os = ...)]` attribute specifying supported system(s).
Please see [Rust reference](https://doc.rust-lang.org/reference/conditional-compilation.html#target_os) for more information.**

It is possible to specify individual file flags for the tests which
require it. They can be specified by appending `FileFlags` variants after a `;` separator,
after (eventual) `root` and features.

```rust,ignore
#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, root, FileSystemFeature::Chflags; FileFlags::SF_IMMUTABLE, FileFlags::UF_IMMUTABLE}
```

Here is a list of the OS which support file flags:

```rust,ignore
{{#include ../../rust/src/test.rs:file_flags_os}}
```

#### Adding features

New features can be added to the `FileSystemFeature` enum.

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
within brackets and with a fat arrow before (`=> [FileType::Regular]`).
The test function should also accept a `FileType` parameter to operate on.

For example:

```rust,ignore
crate::test_case! {change_perm, root, FileSystemFeature::Chflags; FileFlags::SF_IMMUTABLE, FileFlags::UF_IMMUTABLE 
=> [FileType::Regular, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn change_perm(ctx: &mut TestContext, f_type: FileType) {
```

## TODO: Platform-specific functions 

Some functions (like `lchmod`) are not supported on every platform.
