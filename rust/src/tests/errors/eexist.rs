/// Create a test case which asserts that the sycall
/// returns EEXIST if the named file exists.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// eloop_comp_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// eloop_comp_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments, for syscalls
///   requiring to compute other arguments.
///
/// ```
/// eloop_comp_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! eexist_file_exists_test_case {
    ($syscall: ident, $($f: expr),+ $(; $attrs:tt )?) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns EEXIST if the named file exists")]
            eexist_file_exists $(, $attrs )? => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
        }
        fn eexist_file_exists(ctx: &mut crate::context::TestContext,
            ft: crate::context::FileType) {
            let path = ctx.create(ft).unwrap();
            $( assert_eq!($f(ctx, &path), Err(nix::errno::Errno::EEXIST)); )+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        eexist_file_exists_test_case!($syscall, |_: &mut $crate::context::TestContext,
            path: &std::path::Path| {
            $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use eexist_file_exists_test_case;
