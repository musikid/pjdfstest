# Test declaration

Test cases have the same structure than usual Rust tests,
that is `unwrap`ing `Result`s and using assertion macros (`assert` and `assert_eq`),
the exception being that it should take a `&mut TestContext` parameter.
It might also take a `FileType` argument if required.
It also needs an additional declaration with the `test_case!` macro alongside the function,
with the function name being the only mandatory argument.

For example:

```rust,ignore
// chmod/00.t:L58
crate::test_case! {
    /// chmod updates ctime when it succeeds
    update_ctime => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn update_ctime(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    assert_ctime_changed(ctx, &path, || {
        assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_ok());
    });
}
```

All the structures and functions needed are documented in the `pjdfstest` crate,
which you can obtain by running `cargo doc --open` in the `rust` directory
or by visiting the [documentation](/doc/pjdfstest).

## Test context

The [`TestContext`](doc/pjdfstest/context/struct.TestContext.html)
struct is a helper struct which provides methods to create files,
sleep, change user, etc.
It is passed as a parameter to the test functions and should be used
to interact with the system in order to ensure that the tests are isolated
and do not interfere with each other.

### Serialization

When a test case needs to be run in a serialized manner, the
[`SerializedTestContext`](doc/pjdfstest/context/struct.SerializedTestContext.html)
struct should be used instead.
It provides additional methods to change the user, group,
supplementary groups, or umask of the process.
Please see the [Serialized test cases](#serialized-test-cases) section for more information.

## Assertions

The `assert` and `assert_eq` macros should be used to check the results of the tests.
The `assert` macro should be used when the test is checking a condition,
while the `assert_eq` macro should be used when the test is checking
that two values are equal.
In addition to these macros, the suite provides some additional assertion functions which
should be used when appropriate.
The tests [module](doc/pjdfstest/tests/index.html#functions) documentation provides
a list of these functions.

## Description

It is possible to provide doc comments which will be used as documentation for developers
but also be displayed to users when they run the test.
The doc comments should be written in the `test_case!` declaration, before anything.
For example:

```rust,ignore
crate::test_case! {
    /// The file mode of a newly created file should not affect whether
    /// posix_fallocate will work, only the create args
    /// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=154873
    affected_only_create_flags, serialized, root, FileSystemFeature::PosixFallocate
}
```

## Parameterization

It is possible to give additional parameters to the test case macro,
to modify the execution of the tests or add requirements.

### File-system exclusive features

Some features are not available for every file system.
For tests requiring such features, the execution becomes opt-in.
A variant of the `FileSystemFeature` enum corresponding to this feature
should be specified after potential `root` requirement and before guards.
Multiple features can be specified, separated by a comma `,`.

For example:

```rust,ignore
#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, FileSystemFeature::Chflags, FileSystemFeature::PosixFallocate ...}
```

#### Adding features

New features can be added to the `FileSystemFeature` enum.
A description of the feature should be provided as documentation
for both developers and users.

### Guards

It is possible to specify "guards", which are functions which checks if a requirement
is met and return an error if not so the test will be skipped.
They can be specified by appending function names after a `;` separator,
after potential `root` requirement and features.

The function has to take a `&Config` argument
which contains the current configuration
and a `&Path` which represents the parent folder
of the potential test context which would be created.

#### Guard signature

```rust,ignore
/// Function which indicates if the test should be skipped by returning an error.
pub type Guard = fn(&Config, &Path) -> anyhow::Result<()>;
```

#### Example

```rust,ignore
fn has_reasonable_link_max(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    let link_max = pathconf(base_path, nix::unistd::PathconfVar::LINK_MAX)?
        .ok_or_else(|| anyhow::anyhow!("Failed to get LINK_MAX value"))?;

    if link_max >= LINK_MAX_LIMIT {
        anyhow::bail!("LINK_MAX value is too high ({link_max}, expected smaller than {LINK_MAX_LIMIT}");
    }

    Ok(())
}

crate::test_case! {
    /// link returns EMLINK if the link count of the file named by name1 would exceed {LINK_MAX}
    link_count_max; has_reasonable_link_max
}
...
```

### Root privileges

Some tests may need root privileges to run.
To declare that a test function require such privileges,
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
crate::test_case! {change_perm, root, FileSystemFeature::Chflags => [Regular, Fifo, Block, Char, Socket]}
fn change_perm(ctx: &mut TestContext, f_type: FileType) {
```

## Platform-specific features

Some features (like `lchmod`) are not supported on every operating system.
When a test make use of such feature, it is possible to restrain its compilation
to the supported operating systems, with the attribute `#[cfg(feature_name)]`.
It is also possible to apply this attribute on an aspect or even a syscall module.
For example:

```rust,ignore
#[cfg(lchmod)]
mod lchmod;
```

To declare it, the feature and its requirements have to be specified in the `build.rs` file
using the usual conditional compilation
[syntax](https://doc.rust-lang.org/reference/conditional-compilation.html).
Then, the feature should be added to the `cfg_aliases!` macro.
With `lchmod`, we would get:

```rust,ignore
    cfg_aliases! {
        ...
        lchmod: { any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly") },
        ...
    }
```

## Serialized test cases

Some test cases need functions only available when they are run serialized,
especially when they affect the whole process.
An example is changing user (`SerializedTestContext::as_user`).
To have access to these functions, the test should be declared with a
[`SerializedTestContext`](doc/pjdfstest/context/struct.SerializedTestContext.html)
parameter in place of `TestContext` and the `serialized` keyword
should be prepended before features and `root` requirement.

For example:

```rust,ignore
crate::test_case! {
    /// link changes neither ctime of file nor ctime or mtime of parent when it fails
    // link/00.t#77
    unchanged_ctime_fails, serialized, root => [Regular, Fifo, Block, Char, Socket]
}
fn unchanged_ctime_fails(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let new_path = ctx.gen_path();

    let user = ctx.get_new_user();
    assert_times_unchanged()
        .path(&file, CTIME)
        .path(ctx.base_path(), CTIME | MTIME)
        .execute(ctx, false, || {
            ctx.as_user(user, None, || {
                assert!(matches!(
                    link(&file, &new_path),
                    Err(Errno::EPERM | Errno::EACCES)
                ));
            })
        });
}
```
