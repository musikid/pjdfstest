//! Macros for defining test cases.
//! 
//! The `test_case` macro is used to define test cases.

/// The `test_case` macro is used to define test cases for a test suite.
/// 
/// The macro is used to define a test case for a test suite. 
/// The test case can be serialized or non-serialized, require root privileges, and can be run on specific file types.
/// It can also require specific features to be enabled, and have guards which are run before the test case is executed to determine if conditions are met.
/// 
/// The macro supports mutiple parameters which can be combined in a specific order:
/// 
/// ```rust,ignore
/// // Non-serialized test case
/// test_case! {
///    /// description
///   basic
/// }
/// ```
/// 
/// ```rust,ignore
/// // Non-serialized test case with required features, guards, and root privileges
/// test_case! {
///   /// description
///  features, FileSystemFeature::Chflags, FileSystemFeature::PosixFallocate; guard_example, root
/// }
/// ```
macro_rules! test_case {
    ($(#[doc = $docs:expr])*
        $f:ident, serialized, root $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $guards: tt )?) => {
        $crate::test_case! {@serialized $f, &[$( $features ),*], &[$( $( $flags ),+ )?], concat!($($docs),*), true $(=> $guards)?}
    };
    ($(#[doc = $docs:expr])*
        $f:ident, serialized $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $guards: tt )?) => {
        $crate::test_case! {@serialized $f, &[$( $features ),*], &[$( $( $flags ),+ )?], concat!($($docs),*), false $(=> $guards)?}
    };
    ($(#[doc = $docs:expr])*
        $f:ident, root $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $guards: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $( $flags ),+ )?], true, concat!($($docs),*) $(=> $guards)?}
    };
    ($(#[doc = $docs:expr])*
        $f:ident $(,)* $( $features:expr ),* $(,)* $(; $( $flags:expr ),+)? $(=> $guards: tt )?) => {
        $crate::test_case! {@ $f, &[$( $features ),*], &[$( $( $flags ),+ )?], false, concat!($($docs),*) $(=> $guards)?}
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
                        require_root: $require_root || $crate::context::FileType::$file_type $( ($ft_args) )?.privileged(),
                        fun: $crate::test::TestFn::Serialized(|ctx| $f(ctx, $crate::context::FileType::$file_type $( ($ft_args) )?)),
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
                        require_root: $require_root || $crate::context::FileType::$file_type $( ($ft_args) )?.privileged(),
                        fun: $crate::test::TestFn::NonSerialized(|ctx| $f(ctx, $crate::context::FileType::$file_type $( ($ft_args) )?)),
                    }
                }
            }
        )+
    };
}

pub(crate) use test_case;

#[cfg(test)]
mod t {
    use crate::context::FileType;
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
            tc.guards.iter().map(|&g| g as usize).collect::<Vec<_>>(),
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
