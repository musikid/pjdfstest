use std::fmt::Debug;

use linkme::distributed_slice;
use thiserror::Error;

use crate::runner::context::ContextError;
pub use crate::runner::context::TestContext;

pub type TestResult = std::result::Result<(), TestError>;

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
    pub syscall: Option<ExclSyscall>,
}

#[distributed_slice]
pub static TEST_CASES: [TestCase] = [..];

/// Syscalls which are not available on every OS/file system combination.
#[derive(Debug, strum::IntoStaticStr)]
pub enum ExclSyscall {
    PosixFallocate,
}
