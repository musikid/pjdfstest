use thiserror::Error;

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
pub enum TestError {}

/// A group of test cases.
///
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
