#[macro_export]
macro_rules! test_case {
    ($(#[doc = $docs:literal])*
        $f:ident, root $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $( $flags ),+ )?], true, concat!($($docs),*) $(=> $ftypes)?}
    };
    ($(#[doc = $docs:literal])*
        $f:ident $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $( $flags ),+ )?], false, concat!($($docs),*) $(=> $ftypes)?}
    };


    (@ $f:ident, $features:expr, $flags:expr, $require_root:expr, $desc:expr ) => {
        paste::paste! {
            ::inventory::submit! {
                $crate::test::TestCase {
                    name: concat!(module_path!(), "::", stringify!($f)),
                    description: $desc,
                    required_features: $features,
                    required_file_flags: $flags,
                    require_root: $require_root,
                    fun: $f,
                }
            }
        }
    };
    (@ $f:ident, $features:expr, $flags:expr, $require_root:expr, $desc:expr => [$( $file_type:tt $( ($ft_args: tt) )? ),+ $(,)*]) => {
        $(
            paste::paste! {
                ::inventory::submit! {
                    $crate::test::TestCase {
                        name: concat!(module_path!(), "::", stringify!($f), "::", stringify!([<$file_type:lower>])),
                        description: $desc,
                        required_features: $features,
                        required_file_flags: $flags,
                        require_root: $require_root || $crate::runner::context::FileType::$file_type $( ($ft_args) )?.privileged(),
                        fun: |ctx| $f(ctx, $crate::runner::context::FileType::$file_type $( ($ft_args) )?),
                    }
                }
            }
        )+
    };
}
