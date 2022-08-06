use std::{fs::metadata, os::unix::prelude::PermissionsExt};

use nix::{
    sys::stat::{lstat, mode_t, Mode},
    unistd::{chown, mkdir, Gid, Uid, User},
};

use crate::{
    runner::context::{SerializedTestContext, TestContext},
    utils::{chmod, ALLPERMS},
};

use super::assert_time_changed;

crate::test_case! {
    /// POSIX: The file permission bits of the new directory shall be initialized from
    /// mode. These file permission bits of the mode argument shall be modified by the
    /// process' file creation mask.
    permission_bits_from_mode, serialized
}
fn permission_bits_from_mode(ctx: &mut SerializedTestContext) {
    fn assert_perm(ctx: &SerializedTestContext, mkdir_mode: mode_t, expected_mode: mode_t) {
        let path = ctx.gen_path();
        assert!(mkdir(&path, Mode::from_bits_truncate(mkdir_mode)).is_ok());
        let meta = metadata(&path).unwrap();
        assert!(meta.is_dir());
        assert_eq!(
            meta.permissions().mode() as mode_t & ALLPERMS,
            expected_mode
        );
    }

    fn assert_perm_umask(ctx: &SerializedTestContext, mkdir_mode: mode_t, umask: mode_t) {
        ctx.with_umask(umask, || {
            assert_perm(ctx, mkdir_mode, mkdir_mode & (!umask));
        })
    }

    fn assert_perm_mode(ctx: &SerializedTestContext, mkdir_mode: mode_t) {
        assert_perm(ctx, mkdir_mode, mkdir_mode);
    }

    assert_perm_mode(ctx, 0o755);
    assert_perm_mode(ctx, 0o151);
    assert_perm_umask(ctx, 0o151, 0o77);
    assert_perm_umask(ctx, 0o345, 0o70);
    assert_perm_umask(ctx, 0o501, 0o345);
}

crate::test_case! {
    /// POSIX: The directory's user ID shall be set to the process' effective user ID.
    /// The directory's group ID shall be set to the group ID of the parent directory
    /// or to the effective group ID of the process.
    dir_uid_gid_eq_euid_egid, serialized, root
}
fn dir_uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    fn assert_uid_gid(ctx: &SerializedTestContext, user: &User, gid: Option<Gid>) {
        let path = ctx.gen_path();
        ctx.as_user(user, gid.map(|g| vec![g]).as_deref(), || {
            mkdir(&path, Mode::from_bits_truncate(0o755)).unwrap();
        });

        let nix::sys::stat::FileStat {
            st_uid: file_uid,
            st_gid: file_gid,
            ..
        } = lstat(&path).unwrap();
        assert_eq!(file_uid, user.uid.as_raw());
        let egid = gid.unwrap_or(user.gid).as_raw();
        let nix::sys::stat::FileStat {
            st_gid: parent_gid, ..
        } = lstat(ctx.base_path()).unwrap();
        assert!(file_gid == egid || file_gid == parent_gid);
    }

    let user = User::from_uid(Uid::effective()).unwrap().unwrap();
    assert_uid_gid(ctx, &user, None);

    let user = ctx.get_new_user();
    // To check that the entry gid is either parent gid or egid
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();

    let (other_user, group) = ctx.get_new_entry();
    assert_uid_gid(ctx, &user, Some(group.gid));

    chmod(ctx.base_path(), Mode::from_bits_truncate(ALLPERMS)).unwrap();

    let group = ctx.get_new_group();
    assert_uid_gid(ctx, &other_user, Some(group.gid));
}

crate::test_case! {
    /// POSIX: Upon successful completion, mkdir() shall mark for update the st_atime,
    /// st_ctime, and st_mtime fields of the directory. Also, the st_ctime and
    /// st_mtime fields of the directory that contains the new entry shall be marked
    /// for update.
    changed_time_fields_success
}
fn changed_time_fields_success(ctx: &mut TestContext) {
    let uid = Uid::effective();
    let gid = Gid::effective();
    chown(ctx.base_path(), Some(uid), Some(gid)).unwrap();
    let path = ctx.gen_path();
    assert_time_changed(ctx, ctx.base_path(), &path, true, true, false, || {
        mkdir(&path, Mode::from_bits_truncate(0o755)).unwrap();
    });
}
