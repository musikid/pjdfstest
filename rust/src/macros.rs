#[macro_export]
macro_rules! test_case {
    ($f:ident, root, $( $features:expr ),+ $(,)* $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),+], &[], true $(=> $ftypes)?}
    };
    ($f:ident $(,)* $( $features:expr ),* $(,)* ; $( $flags:expr ),+ $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $flags ),+], false $(=> $ftypes)?}
    };
    ($f:ident $(,)* $( $features:expr ),* $(,)* $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[], false $(=> $ftypes)?}
    };


    (@ $f:ident, $features:expr, $flags:expr, $require_root:expr) => {
        paste::paste! {
            #[linkme::distributed_slice($crate::test::TEST_CASES)]
            static [<CASE_$f:upper>]: $crate::test::TestCase = $crate::test::TestCase {
                name: concat!(module_path!(), "::", stringify!($f)),
                required_features: $features,
                required_file_flags: $flags,
                require_root: $require_root,
                fun: $f,
            };
        }
    };
    (@ $f:ident, $features:expr, $flags:expr, $require_root:expr => [$( FileType:: $file_type:tt $( ($ft_args: tt) )? ),+ $(,)*]) => {
        $(
            paste::paste! {
                #[linkme::distributed_slice($crate::test::TEST_CASES)]
                static [<CASE_$f:upper$file_type:upper>]: $crate::test::TestCase = $crate::test::TestCase {
                    name: concat!(module_path!(), "::", stringify!($f), "::", stringify!([<$file_type:lower>]), "_type"),
                    required_features: $features,
                    required_file_flags: $flags,
                    require_root: $require_root || FileType::$file_type $( ($ft_args) )?.privileged(),
                    fun: |ctx| $f(ctx, FileType::$file_type $( ($ft_args) )?),
                };
            }
        )+
    };
}
