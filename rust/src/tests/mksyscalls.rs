//! Builder functions for `mk`-family syscalls tests.

use std::{
    fs::{metadata, FileType},
    os::unix::prelude::PermissionsExt,
    path::Path,
};

use nix::{
    sys::stat::{lstat, mode_t, Mode},
    unistd::{chown, Gid, Uid, User},
};

use crate::{
    runner::context::{SerializedTestContext},
    utils::{chmod, ALLPERMS},
};

/// Assert that the created entry gets its permission bits from the mode provided to the function
/// negated by the process file creation mask (umask), and its file type equal the expected one.
pub(super) fn permission_bits_from_mode_builder<F, T, C>(
    ctx: &mut SerializedTestContext,
    f: F,
    f_type_check: C,
) where
    F: Fn(&Path, Mode) -> nix::Result<T>,
    C: Fn(&FileType) -> bool,
{
    fn assert_perm<F, T, C>(
        ctx: &SerializedTestContext,
        mode: mode_t,
        expected_mode: mode_t,
        f: F,
        f_type_check: C,
    ) where
        F: Fn(&Path, Mode) -> nix::Result<T>,
        C: Fn(&FileType) -> bool,
    {
        let path = ctx.gen_path();
        assert!(f(&path, Mode::from_bits_truncate(mode)).is_ok());
        let meta = metadata(&path).unwrap();
        assert!(f_type_check(&meta.file_type()));
        assert_eq!(
            meta.permissions().mode() as mode_t & ALLPERMS,
            expected_mode
        );
    }

    /// Assert that the created entry permission bits equal `mode AND (NOT umask)`.
    fn assert_perm_umask<F, T, C>(
        ctx: &SerializedTestContext,
        mode: mode_t,
        umask: mode_t,
        f: F,
        check: C,
    ) where
        F: Fn(&Path, Mode) -> nix::Result<T>,
        C: Fn(&FileType) -> bool,
    {
        ctx.with_umask(umask, || {
            assert_perm(ctx, mode, mode & (!umask), f, check);
        })
    }

    /// Assert that the created entry permission bits equal mode.
    fn assert_perm_mode<F, T, C>(ctx: &SerializedTestContext, mode: mode_t, f: F, check: C)
    where
        F: Fn(&Path, Mode) -> nix::Result<T>,
        C: Fn(&FileType) -> bool,
    {
        assert_perm(ctx, mode, mode, f, check);
    }

    assert_perm_mode(ctx, 0o755, &f, &f_type_check);
    assert_perm_mode(ctx, 0o151, &f, &f_type_check);
    assert_perm_umask(ctx, 0o151, 0o77, &f, &f_type_check);
    assert_perm_umask(ctx, 0o345, 0o70, &f, &f_type_check);
    assert_perm_umask(ctx, 0o501, 0o345, f, f_type_check);
}

/// Assert that the entry's user ID is set to the process' effective user ID.
/// The entry's group ID should be set to the group ID of the parent directory
/// or the effective group ID of the process.
pub(super) fn uid_gid_eq_euid_or_parent_uid_egid_builder<F, T>(
    ctx: &mut SerializedTestContext,
    f: F,
) where
    F: Fn(&Path, Mode) -> nix::Result<T>,
{
    fn assert_uid_gid<F, T>(ctx: &SerializedTestContext, user: &User, gid: Option<Gid>, f: F)
    where
        F: Fn(&Path, Mode) -> nix::Result<T>,
    {
        let path = ctx.gen_path();
        ctx.as_user(user, gid.map(|g| vec![g]).as_deref(), || {
            f(&path, Mode::from_bits_truncate(0o755)).unwrap();
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
    assert_uid_gid(ctx, &user, None, &f);

    let user = ctx.get_new_user();
    // To check that the entry gid is either parent gid or egid
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();

    let (other_user, group) = ctx.get_new_entry();
    assert_uid_gid(ctx, &user, Some(group.gid), &f);

    chmod(ctx.base_path(), Mode::from_bits_truncate(ALLPERMS)).unwrap();

    let group = ctx.get_new_group();
    assert_uid_gid(ctx, &other_user, Some(group.gid), f);
}
