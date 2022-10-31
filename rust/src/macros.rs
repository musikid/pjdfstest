#[macro_export]
macro_rules! test_case {
    ($(#[doc = $docs:expr])*
        $f:ident, serialized, root $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $ftypes: tt )?) => {
        $crate::test_case! {@serialized $f, &[$( $features ),*], &[$( $( $flags ),+ )?], concat!($($docs),*), true $(=> $ftypes)?}
    };
    ($(#[doc = $docs:expr])*
        $f:ident, serialized $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $ftypes: tt )?) => {
        $crate::test_case! {@serialized $f, &[$( $features ),*], &[$( $( $flags ),+ )?], concat!($($docs),*), false $(=> $ftypes)?}
    };
    ($(#[doc = $docs:expr])*
        $f:ident, root $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $( $flags ),+ )?], true, concat!($($docs),*) $(=> $ftypes)?}
    };
    ($(#[doc = $docs:expr])*
        $f:ident $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $ftypes: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $( $flags ),+ )?], false, concat!($($docs),*) $(=> $ftypes)?}
    };



    (@serialized $f:ident, $features:expr, $guards:expr, $desc:expr, $require_root:expr ) => {
        ::inventory::submit! {
            $crate::test::TestCase {
                name: concat!(module_path!(), "::", stringify!($f)),
                description: $desc,
                required_features: $features,
                guards: $guards,
                require_root: $require_root,
                fun: $crate::test::TestFn::Serialized($f),
            }
        }
    };
    (@serialized $f:ident, $features:expr, $guards:expr, $desc:expr, $require_root:expr => [$( $file_type:tt $( ($ft_args: tt) )? ),+ $(,)*]) => {
        $(
            paste::paste! {
                ::inventory::submit! {
                    $crate::test::TestCase {
                        name: concat!(module_path!(), "::", stringify!($f), "::", stringify!([<$file_type:lower>])),
                        description: $desc,
                        required_features: $features,
                        guards: $guards,
                        require_root: $require_root || $crate::runner::context::FileType::$file_type $( ($ft_args) )?.privileged(),
                        fun: $crate::test::TestFn::Serialized(|ctx| $f(ctx, $crate::runner::context::FileType::$file_type $( ($ft_args) )?)),
                    }
                }
            }
        )+
    };

    (@ $f:ident, $features:expr, $guards:expr, $require_root:expr, $desc:expr ) => {
        ::inventory::submit! {
            $crate::test::TestCase {
                name: concat!(module_path!(), "::", stringify!($f)),
                description: $desc,
                required_features: $features,
                guards: $guards,
                require_root: $require_root,
                fun: $crate::test::TestFn::NonSerialized($f),
            }
        }
    };
    (@ $f:ident, $features:expr, $guards:expr, $require_root:expr, $desc:expr => [$( $file_type:tt $( ($ft_args: tt) )? ),+ $(,)*]) => {
        $(
            paste::paste! {
                ::inventory::submit! {
                    $crate::test::TestCase {
                        name: concat!(module_path!(), "::", stringify!($f), "::", stringify!([<$file_type:lower>])),
                        description: $desc,
                        required_features: $features,
                        guards: $guards,
                        require_root: $require_root || $crate::runner::context::FileType::$file_type $( ($ft_args) )?.privileged(),
                        fun: $crate::test::TestFn::NonSerialized(|ctx| $f(ctx, $crate::runner::context::FileType::$file_type $( ($ft_args) )?)),
                    }
                }
            }
        )+
    };
}

#[cfg(test)]
mod t {
    use crate::runner::context::FileType;
    use crate::test::FileSystemFeature;
    use crate::{SerializedTestContext, TestCase, TestContext, TestFn};
    use std::path::Path;

    crate::test_case! {
        /// description
        basic
    }
    fn basic(_: &mut TestContext) {}
    #[test]
    fn basic_test() {
        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::basic")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(!tc.require_root);
        assert!(tc.required_features.is_empty());
        assert!(matches!(tc.fun, TestFn::NonSerialized(f) if f as usize == basic as usize));
        assert!(tc.guards.is_empty());
    }

    crate::test_case! {
        /// description
        features, FileSystemFeature::Chflags, FileSystemFeature::PosixFallocate
    }
    fn features(_: &mut TestContext) {}
    #[test]
    fn features_test() {
        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::features")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(!tc.require_root);
        assert_eq!(
            tc.required_features,
            &[
                FileSystemFeature::Chflags,
                FileSystemFeature::PosixFallocate
            ]
        );
        assert!(matches!(tc.fun, TestFn::NonSerialized(f) if f as usize == features as usize));
        assert!(tc.guards.is_empty());
    }

    fn guard_example(_: &crate::config::Config, _: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    crate::test_case! {
        /// description
        guard; guard_example
    }
    fn guard(_: &mut TestContext) {}
    #[test]
    fn guard_test() {
        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::guard")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(!tc.require_root);
        assert_eq!(
            tc.guards
                .iter()
                .map(|&g| g as usize)
                .collect::<Vec<_>>(),
            vec![guard_example as usize]
        );
        assert!(matches!(tc.fun, TestFn::NonSerialized(f) if f as usize == guard as usize));
    }

    crate::test_case! {
        /// description
        root, root
    }
    fn root(_: &mut TestContext) {}
    #[test]
    fn root_test() {
        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::root")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(tc.require_root);
        assert!(tc.required_features.is_empty());
        assert!(matches!(tc.fun, TestFn::NonSerialized(f) if f as usize == root as usize));
        assert!(tc.guards.is_empty());
    }

    crate::test_case! {
        /// description
        file_types => [Regular, Fifo]
    }
    fn file_types(_: &mut TestContext, _: FileType) {}
    #[test]
    fn file_types_test() {
        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::file_types::fifo")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(!tc.require_root);
        assert!(tc.required_features.is_empty());
        assert!(tc.guards.is_empty());
        // Can't check fun because it's a closure

        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::file_types::regular")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(!tc.require_root);
        assert!(tc.required_features.is_empty());
        assert!(tc.guards.is_empty());
        // Can't check fun because it's a closure
    }

    crate::test_case! {
        /// description
        serialized, serialized
    }
    fn serialized(_: &mut SerializedTestContext) {}
    #[test]
    fn serialized_test() {
        let tc = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::macros::t::serialized")
            .unwrap();
        assert_eq!(" description", tc.description);
        assert!(!tc.require_root);
        assert!(tc.required_features.is_empty());
        assert!(matches!(tc.fun, TestFn::Serialized(f) if f as usize == serialized as usize));
        assert!(tc.guards.is_empty());
    }
}
