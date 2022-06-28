use std::fmt::Debug;

use thiserror::Error;

use crate::runner::context::ContextError;
pub use crate::runner::context::TestContext;

pub type TestResult = std::result::Result<(), TestError>;

/// A single test function.
/// Can also be run exclusively on a particular file system.
pub struct Test {
    pub name: &'static str,
    pub fun: fn(&mut TestContext),
    pub require_root: bool,
}

/// Error returned by a test function.
#[derive(Error, Debug)]
pub enum TestError {
    #[error("error while creating file: {0}")]
    CreateFile(ContextError),
    #[error("error while calling syscall: {0}")]
    Nix(#[from] nix::Error),
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

#[derive(Debug, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Syscall {
    Chmod,
}
