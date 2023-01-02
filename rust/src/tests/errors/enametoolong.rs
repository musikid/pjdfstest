/// Create a test case which asserts that the sycall
/// returns `ENAMETOOLONG` if a component of a pathname
/// exceeded `{NAME_MAX}` characters.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```ignore
/// // `unlink` accepts only a path as argument.
/// enametoolong_comp_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// enametoolong_comp_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```ignore
/// enametoolong_comp_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! enametoolong_comp_test_case {
    ($syscall: ident, $($f: expr),+ $(; $attrs:tt )?) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENAMETOOLONG if a component of a pathname",
                 " exceeded {NAME_MAX} characters")]
            enametoolong_component $(, $attrs )?
        }
        fn enametoolong_component(ctx: &mut TestContext) {
            use $crate::context::FileType;
            use nix::errno::Errno;

            let mut invalid_path = ctx.create_name_max(FileType::Regular).unwrap();
            invalid_path.set_extension("x");
            $(assert_eq!($f(ctx, &invalid_path), Err(Errno::ENAMETOOLONG));)+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enametoolong_comp_test_case!($syscall, |_ctx: &mut crate::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use enametoolong_comp_test_case;

/// Create a test case which asserts that the sycall
/// returns `ENAMETOOLONG` if a component of either pathname
/// exceeds `{NAME_MAX}` characters.
/// ```ignore
/// // `rename` accepts two arguments.
/// enametoolong_either_comp_test_case!(rename);
/// ````
macro_rules! enametoolong_either_comp_test_case {
    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENAMETOOLONG if a component of either pathname",
                 " exceeded {NAME_MAX} characters")]
            enametoolong_component
        }
        fn enametoolong_component(ctx: &mut TestContext) {
            use nix::errno::Errno;
            use $crate::context::FileType;

            let mut invalid_path = ctx.create_name_max(FileType::Regular).unwrap();
            invalid_path.set_extension("x");
            let valid_path = ctx.create_name_max(FileType::Regular).unwrap();
            assert_eq!(
                $syscall(&valid_path, &invalid_path),
                Err(Errno::ENAMETOOLONG)
            );
            assert_eq!(
                $syscall(&invalid_path, &valid_path),
                Err(Errno::ENAMETOOLONG)
            );
        }
    };
}

pub(crate) use enametoolong_either_comp_test_case;

/// Create a test case which asserts that the sycall
/// returns `ENAMETOOLONG` if an entire pathname
/// exceeds `{PATH_MAX}` characters.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```ignore
/// // `unlink` accepts only a path as argument.
/// enametoolong_path_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// enametoolong_path_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```ignore
/// enametoolong_path_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! enametoolong_path_test_case {
    ($syscall: ident, $($f: expr),+ $(; $attrs:tt )?) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENAMETOOLONG if an entire pathname",
                 " exceeded {PATH_MAX} characters")]
            enametoolong_path $(, $attrs )?
        }
        fn enametoolong_path(ctx: &mut TestContext) {
            use $crate::context::FileType;
            use nix::errno::Errno;

            let mut invalid_path = ctx.create_path_max(FileType::Regular).unwrap();
            invalid_path.set_extension("x");
            $(assert_eq!($f(ctx, &invalid_path), Err(Errno::ENAMETOOLONG));)+
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enametoolong_path_test_case!($syscall, |_ctx: &mut crate::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use enametoolong_path_test_case;

/// Create a test case which asserts that the sycall
/// returns `ENAMETOOLONG` if an entire pathname
/// exceeds `{PATH_MAX}` characters.
/// ```ignore
/// // `rename` accepts two arguments.
/// enametoolong_either_comp_test_case!(rename);
/// ````
macro_rules! enametoolong_either_path_test_case {
    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 " returns ENAMETOOLONG if an entire pathname",
                 " exceeded {PATH_MAX} characters")]
            enametoolong_path
        }
        fn enametoolong_path(ctx: &mut TestContext) {
            use nix::errno::Errno;
            use $crate::context::FileType;

            let mut invalid_path = ctx.create_path_max(FileType::Regular).unwrap();
            invalid_path.set_extension("x");
            let valid_path = ctx.create_path_max(FileType::Regular).unwrap();
            assert_eq!(
                $syscall(&invalid_path, &valid_path),
                Err(Errno::ENAMETOOLONG)
            );
            assert_eq!(
                $syscall(&invalid_path, &valid_path),
                Err(Errno::ENAMETOOLONG)
            );
        }
    };
}

pub(crate) use enametoolong_either_path_test_case;
