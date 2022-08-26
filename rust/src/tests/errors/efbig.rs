use nix::{errno::Errno, sys::stat::stat, unistd::truncate};

use crate::runner::context::{FileType, TestContext};

crate::test_case! {
    /// truncate must not change the file size if it fails with EFBIG or EINVAL
    /// because the length argument was greater than the maximum file size
    // (f)truncate/12.t
    truncate_efbig
}
fn truncate_efbig(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let size = 999999999999999;
    let res = truncate(&file, size);

    let expected_size = match res {
        Ok(_) => size,
        Err(Errno::EFBIG | Errno::EINVAL) => 0,
        Err(e) => panic!("truncate failed with {e}"),
    };

    let f_stat = stat(&file).unwrap();
    assert_eq!(f_stat.st_size, expected_size);
}
