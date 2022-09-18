use std::{fs::FileType as StdFileType, os::unix::fs::FileTypeExt, path::Path};

use nix::errno::Errno;
use nix::sys::stat::{mknod, Mode, SFlag};

use crate::runner::context::{FileType, SerializedTestContext, TestContext};

use super::errors::enotdir::enotdir_comp_test_case;
use super::mksyscalls::{assert_perms_from_mode_and_umask, assert_uid_gid};
use super::{assert_times_changed, ATIME, CTIME, MTIME};

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
    assert_perms_from_mode_and_umask(ctx, mknod_wrapper, StdFileType::is_fifo);
}

crate::test_case! {
    /// POSIX: The FIFO's user ID shall be set to the process' effective user ID.
    /// The FIFO's group ID shall be set to the group ID of the parent directory or to
    /// the effective group ID of the process.
    uid_gid_eq_euid_egid, serialized, root
}
fn uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    assert_uid_gid(ctx, mknod_wrapper);
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

// mknod/01.t
enotdir_comp_test_case!(mknod(~path, SFlag::S_IFIFO, Mode::empty(), 0));

crate::test_case! {
    /// mknod returns ENOTDIR if a component of the path prefix is not a directory
    /// when trying to create char/block files
    enotdir_comp_char_block, root => [Regular, Fifo, Block, Char, Socket]
}
fn enotdir_comp_char_block(ctx: &mut TestContext, ft: FileType) {
    let base_path = ctx.create(ft).unwrap();
    let path = base_path.join("previous_not_dir");

    // mknod/01.t
    assert_eq!(
        mknod(&path, SFlag::S_IFCHR, Mode::empty(), 0).unwrap_err(),
        Errno::ENOTDIR
    );
    assert_eq!(
        mknod(&path, SFlag::S_IFBLK, Mode::empty(), 0).unwrap_err(),
        Errno::ENOTDIR
    );
}
