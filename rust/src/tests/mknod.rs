use std::{fs::FileType, os::unix::fs::FileTypeExt, path::Path};

use nix::sys::stat::{mknod, Mode, SFlag};

use crate::runner::context::{SerializedTestContext, TestContext};

use super::{assert_times_changed, ATIME, CTIME, MTIME};
use super::mksyscalls::{
    permission_bits_from_mode_builder,
    uid_gid_eq_euid_or_parent_uid_egid_builder,
};

/// TODO: Shouldn't test it for other types?
fn mknod_wrapper(path: &Path, mode: Mode) -> nix::Result<()> {
    mknod(path, SFlag::S_IFIFO, mode, 0)
}

crate::test_case! {
    /// POSIX: The file permission bits of the new FIFO shall be initialized from
    /// mode. The file permission bits of the mode argument shall be modified by the
    /// process' file creation mask.
    permission_bits_from_mode, serialized
}
fn permission_bits_from_mode(ctx: &mut SerializedTestContext) {
    permission_bits_from_mode_builder(ctx, mknod_wrapper, FileType::is_fifo);
}

crate::test_case! {
    /// POSIX: The FIFO's user ID shall be set to the process' effective user ID.
    /// The FIFO's group ID shall be set to the group ID of the parent directory or to
    /// the effective group ID of the process.
    uid_gid_eq_euid_egid, serialized, root
}
fn uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    uid_gid_eq_euid_or_parent_uid_egid_builder(ctx, mknod_wrapper);
}

crate::test_case! {
    /// POSIX: Upon successful completion, mkfifo() shall mark for update the st_atime,
    /// st_ctime, and st_mtime fields of the file. Also, the st_ctime and
    /// st_mtime fields of the directory that contains the new entry shall be marked
    /// for update.
    changed_time_fields_success
}
fn changed_time_fields_success(ctx: &mut TestContext) {
    let path = ctx.gen_path();

    assert_times_changed()
        .path(ctx.base_path(), CTIME | MTIME)
        .paths(ctx.base_path(), &path, ATIME | CTIME | MTIME)
        .execute(ctx, false, || {
            mknod_wrapper(&path, Mode::from_bits_truncate(0o644)).unwrap();
        });
}
