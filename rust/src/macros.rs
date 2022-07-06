#[macro_export]
macro_rules! test_case {
    ( $f:ident, root, $syscall:path $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, Some($syscall), true $(=> $ftypes)?}
    };
    ( $f:ident, root $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, None, true $(=> $ftypes)?}
    };
    ( $f:ident, $syscall:path $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, Some($syscall), false $(=> $ftypes)?}
    };
    ( $f:ident $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, None, false $(=> $ftypes)?}
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
    ( $f:ident, $syscall:expr, $require_root:expr => [$(FileType::$file_type: tt $(($ft_args: tt))?),+ $(,)*]) => {
        $(
            paste::paste! {
                #[linkme::distributed_slice($crate::test::TEST_CASES)]
                static [<CASE_$f:upper$file_type:upper>]: $crate::test::TestCase = crate::test::TestCase {
                    name: concat!(module_path!(), "::", stringify!($f), "::", stringify!([<$file_type:lower>]), "_type"),
                    syscall: $syscall,
                    require_root: $require_root || FileType::$file_type $( ($ft_args ) )?.privileged(),
                    fun: |ctx| $f(ctx, FileType::$file_type $( ($ft_args ) )?),
                };
            }
        )+
    };
}
