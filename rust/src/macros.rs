#[macro_export]
macro_rules! test_case {
    ( $f:path, root, $syscall:path ) => {
        $crate::test_case!{$f, Some($syscall), true}
    };
    ( $f:path, $syscall:path ) => {
        $crate::test_case!{$f, Some($syscall), false}
    };
    ( $f:path ) => {
        $crate::test_case!{$f, None, false}
    };
    ( $f:path, $syscall:expr, $require_root:expr ) => {
        ::inventory::submit! {
            crate::test::TestCase {
                name: concat!(module_path!(), "::", stringify!($f)),
                syscall: $syscall,
                require_root: $require_root,
                fun: $f
            }
        }
    };
}
