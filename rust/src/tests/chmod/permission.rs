use std::{thread::sleep, time::Duration};

use crate::{
    pjdfs_test_case,
    runner::context::FileType,
    test::{TestContext, TestError, TestResult},
    test_assert,
    tests::chmod::chmod,
};
use nix::sys::stat::{stat, Mode};
use strum::IntoEnumIterator;

// chmod/00.t:L58
fn test_ctime(ctx: &mut TestContext) -> TestResult {
    for f_type in FileType::iter().filter(|&ft| ft == FileType::Symlink) {
        let path = ctx.create(f_type).map_err(TestError::CreateFile)?;
        let ctime_before = stat(&path)?.st_ctime;

        sleep(Duration::from_secs(1));

        chmod(&path, Mode::from_bits_truncate(0o111))?;

        let ctime_after = stat(&path)?.st_ctime;
        test_assert!(ctime_after > ctime_before);
    }

    Ok(())
}

pjdfs_test_case!(permission, { test: test_ctime });
