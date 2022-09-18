/// Create a test case which asserts that the syscall returns ENOTDIR
/// if a component of the path prefix is not a directory.
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
/// ```
macro_rules! enotdir_comp_test_case {
    ($syscall: ident, $f: expr) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENOTDIR if a component of the path prefix is not a directory")]
            enotdir_component => [Regular, Fifo, Block, Char, Socket]
        }
        fn enotdir_component(ctx: &mut crate::runner::context::TestContext,
                            ft: crate::runner::context::FileType) {
            let base_path = ctx.create(ft.clone()).unwrap();
            let path = base_path.join("previous_not_dir");

            assert_eq!($f(ctx, &path).unwrap_err(), nix::errno::Errno::ENOTDIR)
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enotdir_comp_test_case!($syscall, |_ctx: &mut crate::runner::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

/// Create a test case which asserts that the syscall returns ENOTDIR
/// if a component of either path prefix is not a directory.
/// It takes the syscall as its only argument.
/// ```
/// enotdir_comp_either_test_case!(rename);
/// ```
macro_rules! enotdir_comp_either_test_case {
    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENOTDIR if a component of either path prefix is not a directory")]
            enotdir_component_either => [Regular, Fifo, Block, Char, Socket]
        }
        fn enotdir_component_either(
            ctx: &mut crate::runner::context::TestContext,
            ft: crate::runner::context::FileType,
        ) {
            let file = ctx.create(ft.clone()).unwrap();
            let path = file.join("previous_not_dir");
            let new_path = ctx.gen_path();

            assert_eq!($syscall(&*path, &*new_path).unwrap_err(), Errno::ENOTDIR);

            let new_base_path = ctx.create(ft.clone()).unwrap();
            let new_path = new_base_path.join("previous_not_dir");

            assert_eq!($syscall(&*file, &*new_path).unwrap_err(), Errno::ENOTDIR);
        }
    };
}

pub(crate) use enotdir_comp_either_test_case;
pub(crate) use enotdir_comp_test_case;
