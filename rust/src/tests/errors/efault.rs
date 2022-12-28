/// Create a test case which asserts that the sycall
/// returns EFAULT if the path argument points
/// outside the process's allocated address space.
/// It takes a function with the path pointer as argument.
///
/// ```ignore
/// efault_path_test_case!(|ptr| nix::libc::mkdir(ptr, 0o755))
/// ````
macro_rules! efault_path_test_case {
    ($syscall: ident, $fn: expr) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns EFAULT if the path argument points",
            " outside the process's allocated address space"
            )]
            efault_path
        }
        fn efault_path(_: &mut crate::runner::context::TestContext) {
            let f = |ptr| unsafe { $fn(ptr) };

            let null_ptr = std::ptr::null();
            // TODO: This theorically could be a valid pointer, but
            // it's unlikely to be the case in practice. Should we
            // use a more robust way to get a pointer outside the
            // process's allocated address space, though?
            let invalid_ptr = usize::MAX as *const _;

            assert_eq!(
                nix::errno::Errno::result(f(null_ptr)),
                Err(nix::errno::Errno::EFAULT)
            );
            assert_eq!(
                nix::errno::Errno::result(f(invalid_ptr)),
                Err(nix::errno::Errno::EFAULT)
            );
        }
    };
}

pub(crate) use efault_path_test_case;

/// Create a test case which asserts that the sycall
/// returns EFAULT if one of the pathnames specified
/// is outside the process's allocated address space.
///
/// ```ignore
/// efault_error_test_case!(link, nix::libc::link)
/// ```
macro_rules! efault_either_test_case {
    ($syscall:ident, $fn: expr) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns EFAULT if one of the pathnames specified",
            " is outside the process's allocated address space"
            )]
            efault_either
        }
        fn efault_either(ctx: &mut crate::runner::context::TestContext) {
            use nix::NixPath;

            let file = ctx
                .create(crate::runner::context::FileType::Regular)
                .unwrap();

            let null_ptr = std::ptr::null();

            // TODO: This theorically could be a valid pointer, but
            // it's unlikely to be the case in practice. Should we
            // use a more robust way to get a pointer outside the
            // process's allocated address space, though?
            let invalid_ptr = usize::MAX as *const _;

            file.with_nix_path(|cstr| {
                let ptr = cstr.as_ptr();

                assert_eq!(
                    nix::errno::Errno::result(unsafe { $fn(null_ptr, ptr) }),
                    Err(nix::errno::Errno::EFAULT)
                );
                assert_eq!(
                    nix::errno::Errno::result(unsafe { $fn(invalid_ptr, ptr) }),
                    Err(nix::errno::Errno::EFAULT)
                );

                assert_eq!(
                    nix::errno::Errno::result(unsafe { $fn(ptr, null_ptr) }),
                    Err(nix::errno::Errno::EFAULT)
                );
                assert_eq!(
                    nix::errno::Errno::result(unsafe { $fn(ptr, invalid_ptr) }),
                    Err(nix::errno::Errno::EFAULT)
                );
            })
            .unwrap();

            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(invalid_ptr, null_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
            assert_eq!(
                nix::errno::Errno::result(unsafe { $fn(null_ptr, invalid_ptr) }),
                Err(nix::errno::Errno::EFAULT)
            );
        }
    };
}

pub(crate) use efault_either_test_case;
