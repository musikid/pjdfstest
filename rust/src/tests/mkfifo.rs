use std::{fs::FileType, os::unix::fs::FileTypeExt};

use nix::{sys::stat::Mode, unistd::mkfifo};

use crate::runner::context::{SerializedTestContext, TestContext};

use super::mksyscalls::{assert_perms_from_mode_and_umask, assert_uid_gid};
use super::{assert_times_changed, ATIME, CTIME, MTIME};

crate::test_case! {
    /// POSIX: The file permission bits of the new FIFO shall be initialized from
    /// mode. The file permission bits of the mode argument shall be modified by the
    /// process' file creation mask.
    permission_bits_from_mode, serialized
}
fn permission_bits_from_mode(ctx: &mut SerializedTestContext) {
    assert_perms_from_mode_and_umask(ctx, mkfifo, FileType::is_fifo);
}

crate::test_case! {
    /// POSIX: The FIFO's user ID shall be set to the process' effective user ID.
    /// The FIFO's group ID shall be set to the group ID of the parent directory or to
    /// the effective group ID of the process.
    uid_gid_eq_euid_egid, serialized, root
}
fn uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    assert_uid_gid(ctx, mkfifo);
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
            mkfifo(&path, Mode::from_bits_truncate(0o600)).unwrap();
        });
}

// mkfifo/09.t
crate::eexist_test_case! {mkfifo, |_ctx, path|
    mkfifo(path, Mode::from_bits_truncate(0o600))
}
