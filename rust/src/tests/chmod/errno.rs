use nix::{
    errno::Errno,
    sys::stat::{stat, Mode},
};

use crate::{runner::context::FileType, test::TestContext, utils::chmod};

#[cfg(target_os = "freebsd")]
use crate::test::{FileFlags, FileSystemFeature};

#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, FileSystemFeature::Chflags; FileFlags::SF_IMMUTABLE}
#[cfg(target_os = "freebsd")]
fn eperm_immutable_flag(ctx: &mut TestContext) {
    let _path = ctx.create(FileType::Regular).unwrap();
    //TODO: Complete
}
