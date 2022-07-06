#[macro_export]
macro_rules! test_case {
    ( $f:ident, root, $syscall:path ) => {
        $crate::test_case! {$f, Some($syscall), true}
    };
    ( $f:ident, $syscall:path ) => {
        $crate::test_case! {$f, Some($syscall), false}
    };
    ( $f:ident ) => {
        $crate::test_case! {$f, None, false}
    };
    ( $f:ident, $syscall:expr, $require_root:expr ) => {
        paste::paste! {
            #[linkme::distributed_slice($crate::test::TEST_CASES)]
            static [<CASE_$f:upper>]: $crate::test::TestCase = crate::test::TestCase {
                name: concat!(module_path!(), "::", stringify!($f)),
                syscall: $syscall,
                require_root: $require_root,
                fun: $f,
            };
        }
    };
}
