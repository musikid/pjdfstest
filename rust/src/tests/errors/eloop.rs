use std::path::PathBuf;

use crate::context::{FileType, TestContext};

/// Create a loop between two symbolic links and return them.
pub fn create_loop_symlinks(ctx: &mut TestContext) -> (PathBuf, PathBuf) {
    let loop1 = ctx.gen_path();
    let loop2 = ctx.gen_path();

    (
        ctx.new_file(FileType::Symlink(Some(loop2.clone())))
            .name(&loop1)
            .create()
            .unwrap(),
        ctx.new_file(FileType::Symlink(Some(loop1)))
            .name(&loop2)
            .create()
            .unwrap(),
    )
}

/// Create a test case which asserts that the sycall
/// returns ELOOP if too many symbolic links were encountered in translating
/// a component of the pathname which is not the last one.
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
macro_rules! eloop_comp_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns ELOOP if too many symbolic",
            " links were encountered in translating a component of the pathname",
            " which is not the last one")]
            eloop_comp
        }
        fn eloop_comp(ctx: &mut crate::context::TestContext) {
            let (mut loop1, mut loop2) = $crate::tests::errors::eloop::create_loop_symlinks(ctx);
            loop1.push("test");
            loop2.push("test");

            $(
                assert_eq!($f(ctx, &loop1).unwrap_err(), nix::errno::Errno::ELOOP);
                assert_eq!($f(ctx, &loop2).unwrap_err(), nix::errno::Errno::ELOOP);
            )+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        eloop_comp_test_case!($syscall, |_: &mut $crate::context::TestContext,
            path: &std::path::Path| {
            $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use eloop_comp_test_case;

/// Create a test case which asserts that the sycall
/// returns ELOOP if too many symbolic links were encountered in translating
/// a component of either pathname which is not the last one.
/// ```ignore
/// eloop_either_test_case!(rename);
/// ```
macro_rules! eloop_either_test_case {
    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns ELOOP if too many symbolic",
            " links were encountered in translating a component of either pathname",
            " which is not the last one")]
            eloop_comp
        }
        fn eloop_comp(ctx: &mut crate::context::TestContext) {
            let (mut loop1, mut loop2) = $crate::tests::errors::eloop::create_loop_symlinks(ctx);
            loop1.push("test");
            loop2.push("test");
            let valid_path = ctx.create(crate::context::FileType::Regular).unwrap();

            assert_eq!(
                $syscall(&loop1.join("test"), &valid_path).unwrap_err(),
                Errno::ELOOP
            );
            assert_eq!(
                $syscall(&loop2.join("test"), &valid_path).unwrap_err(),
                Errno::ELOOP
            );
            assert_eq!(
                $syscall(&valid_path, &loop1.join("test")).unwrap_err(),
                Errno::ELOOP
            );
            assert_eq!(
                $syscall(&valid_path, &loop2.join("test")).unwrap_err(),
                Errno::ELOOP
            );
        }
    };
}

pub(crate) use eloop_either_test_case;

/// Create a test case which asserts that the sycall
/// returns ELOOP if too many symbolic links were encountered in translating
/// the last component of the pathname.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// eloop_final_comp_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// eloop_final_comp_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which can take multiple functions
///   with the context and the path as arguments, for syscalls
///   requring to compute other arguments.
///
/// ```
/// eloop_final_comp_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! eloop_final_comp_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns ELOOP if too many symbolic",
            " links were encountered in translating",
            " the last component of the pathname")]
            eloop_final_comp
        }
        fn eloop_final_comp(ctx: &mut crate::context::TestContext) {
            let (loop1, loop2) = $crate::tests::errors::eloop::create_loop_symlinks(ctx);

            $(
                assert_eq!($f(ctx, &loop1).unwrap_err(), nix::errno::Errno::ELOOP);
                assert_eq!($f(ctx, &loop2).unwrap_err(), nix::errno::Errno::ELOOP);
            )+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        eloop_final_comp_test_case!($syscall, |_: &mut $crate::context::TestContext,
            path: &std::path::Path| {
            $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use eloop_final_comp_test_case;
