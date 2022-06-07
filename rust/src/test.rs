use std::fmt::Debug;

use thiserror::Error;

use crate::runner::context::ContextError;
pub use crate::runner::context::TestContext;

pub type TestResult = std::result::Result<(), TestError>;

/// A single test function.
/// Can also be run exclusively on a particular file system.
pub struct Test {
    pub name: &'static str,
    pub fun: fn(&mut TestContext) -> TestResult,
    pub file_system: Option<String>,
}

/// Error returned bu a test function.
#[derive(Error, Debug)]
pub enum TestError {
    #[error("error while creating file")]
    CreateFile(ContextError),
    #[error("error while calling syscall")]
    Nix(#[from] nix::Error),
    #[error("assertion failed in file {file} at {line}:{column}")]
    FailedAssertion {
        file: &'static str,
        line: u32,
        column: u32,
    },
    #[error(
        "assertion failed in file {file} at {line}:{column} (left == right)
left: {left:#?}
right: {right:#?}"
    )]
    FailedEqualAssertion {
        file: &'static str,
        line: u32,
        column: u32,
        left: Box<dyn Debug>,
        right: Box<dyn Debug>,
    },
}

/// A group of test cases.
pub struct TestGroup {
    pub name: &'static str,
    pub test_cases: &'static [TestCase],
    pub syscall: Option<Syscall>,
}

/// A test case, which is made of multiple test functions.
pub struct TestCase {
    pub name: &'static str,
    pub tests: &'static [Test],
}

#[derive(Debug)]
pub enum Syscall {
    Chmod,
}

#[macro_export]
macro_rules! test_assert {
    ($boolean: expr) => {
        if !$boolean {
            return Err(TestError::FailedAssertion {
                file: file!(),
                line: line!(),
                column: column!(),
            });
        }
    };
}

#[macro_export]
macro_rules! test_assert_eq {
    ($a: expr, $b: expr) => {
        if $a != $b {
            return Err(TestError::FailedEqualAssertion {
                file: file!(),
                line: line!(),
                column: column!(),
                left: Box::new($a),
                right: Box::new($b)
            });
        }
    };
}
