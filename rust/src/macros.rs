/// Create a group of tests, by creating/exporting a `tests` variable,
/// which can then be used with the test runner.
#[macro_export]
macro_rules! pjdfs_group {
    ($name:ident; $syscall:path; $( $group:path ),* $(,)*) => {
       #[allow(non_snake_case, non_upper_case_globals)]
       pub const tests: $crate::test::TestGroup = $crate::test::TestGroup {
            name: stringify!($name),
            syscall: $syscall,
            test_cases: &[
                $( $group ),*
            ]
        };
    };

    ($name:ident; $( $group:path ),* $(,)*) => {
       #[allow(non_snake_case, non_upper_case_globals)]
       pub const tests: $crate::test::TestGroup = $crate::test::TestGroup {
            name: stringify!($name),
            syscall: None,
            test_cases: &[
                $( $group ),*
            ]
        };
    };
}

/// Create a test case, which is made of multiple test functions.
/// An optional argument for executing exclusively on a particular file system can be provided.
#[macro_export]
macro_rules! pjdfs_test_case {
    ($name:path, $( 
                    { test: $test:path 
                    $( , file_system: $fs:path )? 
                    $(, require_root: $require_root: expr)? 
                    } 
                ),+ $(,)*) => {
       #[allow(non_snake_case, non_upper_case_globals)]
        pub const test_case: $crate::test::TestCase = $crate::test::TestCase {
            name: stringify!($name),
            tests: &[
                $( 
                    $crate::pjdfs_test!({
                        test: $test
                        $(, file_system: $fs )? 
                        $(, require_root: $require_root)? 
                    }) 
                ),+
            ]
        };
    };
}

/// Create a test function.
/// An optional argument for executing exclusively on a particular file system can be provided.
#[macro_export]
macro_rules! pjdfs_test {
    ({ test: $test: path }) => {
        $crate::test::Test {
            name: stringify!($test),
            fun: $test,
            file_system: None,
            require_root: false,
        }
    };

    ({ test: $test: path, file_system: $file_system: path }) => {
        $crate::test::Test {
            name: stringify!($test),
            fun: $test,
            file_system: Some($file_system),
            require_root: false,
        }
    };

    ({ test: $test: path, require_root: $require_root: expr }) => {
        $crate::test::Test {
            name: stringify!($test),
            fun: $test,
            file_system: None,
            require_root: $require_root,
        }
    };

    ({ test: $test: path, file_system: $file_system: path, require_root: $require_root: expr }) => {
        $crate::test::Test {
            name: stringify!($test),
            fun: $test,
            file_system: Some($file_system),
            require_root: $require_root,
        }
    };
}
