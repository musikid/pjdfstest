/// Create a test case which asserts that the sycall
/// returns ENOENT if the named file does not exist.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// enoent_named_file_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// enoent_named_file_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```
/// enoent_named_file_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! enoent_named_file_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENOENT if the named file does not exist")]
           enoent_named_file
        }
        fn enoent_named_file(ctx: &mut crate::context::TestContext) {
            let dir = ctx.create(crate::context::FileType::Dir).unwrap();
            let path = dir.join("not_existent");

            $( assert_eq!($f(ctx, &path).unwrap_err(), nix::errno::Errno::ENOENT) );+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enoent_named_file_test_case!($syscall, |_ctx: &mut crate::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use enoent_named_file_test_case;

/// Create a test case which asserts that the sycall
/// returns ENOENT if either of the named file does not exist.
/// ```
/// enoent_either_named_file_test_case!(rename);
/// ```
macro_rules! enoent_either_named_file_test_case {
    ($syscall: ident) => {
        $crate::tests::errors::enoent::enoent_named_file_test_case!(
            $syscall,
            |ctx: &mut crate::context::TestContext, path: &std::path::Path| {
                let real_file = ctx.create(crate::context::FileType::Regular).unwrap();
                let path = path.join("test");
                $syscall(&path, &real_file)
            },
            |ctx: &mut crate::context::TestContext, path: &std::path::Path| {
                let real_file = ctx.create(crate::context::FileType::Regular).unwrap();
                let path = path.join("test");
                $syscall(&*real_file, &path)
            }
        );
    };
}

pub(crate) use enoent_either_named_file_test_case;

/// Create a test case which asserts that the sycall
/// returns ENOENT if the symlink target named file does not exist.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// enoent_symlink_named_file_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// enoent_symlink_named_file_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```
/// enoent_symlink_named_file_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! enoent_symlink_named_file_test_case {
    ($syscall: ident, $f: expr) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENOENT if the symlink target named file does not exist")]
           enoent_symlink
        }
        fn enoent_symlink(ctx: &mut crate::context::TestContext) {
            let dir = ctx.create(crate::context::FileType::Dir).unwrap();
            let path = dir.join("not_existent");
            let link = ctx
                .create(crate::context::FileType::Symlink(Some(path.to_path_buf())))
                .unwrap();

            assert_eq!($f(ctx, &link).unwrap_err(), nix::errno::Errno::ENOENT)
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enoent_symlink_named_file_test_case!($syscall, |_ctx: &mut crate::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use enoent_symlink_named_file_test_case;

/// Create a test case which asserts that the sycall
/// returns ENOENT if a component of the path prefix does not exist.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// enotdir_comp_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// enotdir_comp_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```
/// enotdir_comp_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! enoent_comp_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENOENT if a component of the path prefix does not exist")]
           enoent_comp
        }
        fn enoent_comp(ctx: &mut crate::context::TestContext) {
            let dir = ctx.create(crate::context::FileType::Dir).unwrap();
            let path = dir.join("not_existent").join("test");

            $( assert_eq!($f(ctx, &path).unwrap_err(), nix::errno::Errno::ENOENT) );+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enoent_comp_test_case!($syscall, |_ctx: &mut crate::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use enoent_comp_test_case;
