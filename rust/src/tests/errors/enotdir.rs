/// Asserts that it returns ENOTDIR if a component of the path prefix is not a directory.
macro_rules! assert_enotdir_comp {
    ($syscall: ident, either) => {
        assert_enotdir_comp!($syscall, |ctx: &mut crate::runner::context::TestContext,
                                             path: &std::path::Path| {
                let new_path = ctx.gen_path();
                let file = path.parent().unwrap();
                $syscall(path, &new_path).or($syscall(file, path))
        });
    };

    ($syscall: ident, $f: expr) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                 "returns ENOTDIR if a component of the path prefix is not a directory")]
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
        assert_enotdir_comp!($syscall, |_ctx: &mut crate::runner::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

// To not pollute crate namespace
pub(crate) use assert_enotdir_comp;
