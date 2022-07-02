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
    pub file_system: Option<String>,
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

/// A single minimal test case
pub struct TestCase {
    pub name: &'static str,
    pub require_root: bool,
    pub fun: fn(&mut TestContext),
    pub syscall: Option<Syscall>
}

#[derive(Debug)]
pub enum Syscall {
    Chmod,
}

inventory::collect!{TestCase}
