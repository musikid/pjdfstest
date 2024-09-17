use std::{fs::FileType, os::unix::fs::FileTypeExt};

use nix::{sys::stat::Mode, unistd::mkfifo};

use crate::context::{SerializedTestContext, TestContext};

use super::errors::eexist::eexist_file_exists_test_case;
use super::errors::efault::efault_path_test_case;
use super::errors::eloop::eloop_comp_test_case;
use super::errors::enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case};
use super::errors::enoent::enoent_comp_test_case;
use super::errors::enotdir::enotdir_comp_test_case;
use super::errors::erofs::erofs_new_file_test_case;
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

// mkfifo/01.t
enotdir_comp_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/02.t
enametoolong_comp_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/03.t
enametoolong_path_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/04.t
enoent_comp_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/07.t
eloop_comp_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/08.t
erofs_new_file_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/09.t
eexist_file_exists_test_case!(mkfifo(~path, Mode::empty()));

// mkfifo/12.t
efault_path_test_case!(mkfifo, |ptr| nix::libc::mkfifo(ptr, 0o644));

#[cfg(file_flags)]
mod flag {
    use std::fs::metadata;

    use super::*;
    use crate::tests::errors::eperm::flag::immutable_parent_test_case;

    // mkfifo/10.t
    immutable_parent_test_case!(
        mkfifo,
        |path| mkfifo(path, Mode::from_bits_truncate(0o644)),
        |path| metadata(path).map_or(false, |m| m.file_type().is_fifo())
    );
}
