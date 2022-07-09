#[macro_export]
macro_rules! test_case {
    ( $f:path, root, $syscall:path $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, Some($syscall), true $(=> $ftypes)?}
    };
    ( $f:path, root $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, None, true $(=> $ftypes)?}
    };
    ( $f:path, $syscall:path $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, Some($syscall), false $(=> $ftypes)?}
    };
    ( $f:path $(=> $ftypes: tt )?) => {
        $crate::test_case! {$f, None, false $(=> $ftypes)?}
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
    ( $f:path, $syscall:expr, $require_root:expr => [$( FileType:: $file_type:tt $( ($ft_args: tt) )? ),+ $(,)*]) => {
        $(
            ::inventory::submit! {
                crate::test::TestCase {
                    name: concat!(module_path!(), "::", stringify!($f), "::", stringify!([<$file_type:lower>]), "_type"),
                    syscall: $syscall,
                    require_root: $require_root || FileType::$file_type $( ($ft_args) )?.privileged(),
                    fun: |ctx| $f(ctx, FileType::$file_type $( ($ft_args) )?),
                }
            }
        )+
    };
}
