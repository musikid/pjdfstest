use std::fs::FileType;

use nix::{sys::stat::Mode, unistd::mkdir};

use crate::context::{SerializedTestContext, TestContext};

use super::errors::eexist::eexist_file_exists_test_case;
use super::errors::eloop::eloop_comp_test_case;
use super::errors::enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case};
use super::errors::enoent::enoent_comp_test_case;
use super::mksyscalls::{assert_perms_from_mode_and_umask, assert_uid_gid};
use super::{assert_times_changed, errors::enotdir::enotdir_comp_test_case, ATIME, CTIME, MTIME};

crate::test_case! {
    /// POSIX: The file permission bits of the new directory shall be initialized from
    /// mode. These file permission bits of the mode argument shall be modified by the
    /// process' file creation mask.
    permission_bits_from_mode, serialized
}
fn permission_bits_from_mode(ctx: &mut SerializedTestContext) {
    assert_perms_from_mode_and_umask(ctx, mkdir, FileType::is_dir);
}

crate::test_case! {
    /// POSIX: The directory's user ID shall be set to the process' effective user ID.
    /// The directory's group ID shall be set to the group ID of the parent directory
    /// or to the effective group ID of the process.
    uid_gid_eq_euid_egid, serialized, root
}
fn uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    assert_uid_gid(ctx, mkdir);
}

crate::test_case! {
    /// POSIX: Upon successful completion, mkdir() shall mark for update the st_atime,
    /// st_ctime, and st_mtime fields of the directory. Also, the st_ctime and
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
            mkdir(&path, Mode::from_bits_truncate(0o755)).unwrap();
        });
}

enotdir_comp_test_case!(mkdir(~path, Mode::empty()));

// mkdir/02.t
enametoolong_comp_test_case!(mkdir(~path, Mode::empty()));

// mkdir/03.t
enametoolong_path_test_case!(mkdir(~path, Mode::empty()));

// mkdir/04.t
enoent_comp_test_case!(mkdir(~path, Mode::empty()));

// mkdir/07.t
eloop_comp_test_case!(mkdir(~path, Mode::empty()));

// mkdir/10.t
eexist_file_exists_test_case!(mkdir(~path, Mode::empty()));
