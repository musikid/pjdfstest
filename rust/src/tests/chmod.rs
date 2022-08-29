use std::{fs::metadata, os::unix::prelude::PermissionsExt};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    test::TestContext,
    tests::{assert_ctime_changed, assert_ctime_unchanged},
    utils::{chmod, lchmod, ALLPERMS},
};
use nix::{
    libc::mode_t,
    sys::stat::{lstat, stat, Mode},
    unistd::chown,
};

mod errno;

const ALLPERMS_SUID_SGID: mode_t = ALLPERMS | Mode::S_ISUID.bits() | Mode::S_ISGID.bits();

// chmod/00.t:L24
crate::test_case! {
    /// chmod successfully change permissions
    change_perm => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn change_perm(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type.clone()).unwrap();
    let expected_mode = Mode::from_bits_truncate(0o111);

    chmod(&path, expected_mode).unwrap();

    let actual_mode = stat(&path).unwrap().st_mode;

    assert_eq!(actual_mode & ALLPERMS, expected_mode.bits());

    // We test if it applies through symlinks
    let symlink_path = ctx.create(FileType::Symlink(Some(path.clone()))).unwrap();
    let link_mode = lstat(&symlink_path).unwrap().st_mode;
    let expected_mode = Mode::from_bits_truncate(0o222);

    chmod(&symlink_path, expected_mode).unwrap();

    let actual_mode = stat(&path).unwrap().st_mode;
    let actual_sym_mode = stat(&symlink_path).unwrap().st_mode;
    assert_eq!(actual_mode & ALLPERMS, expected_mode.bits());
    assert_eq!(actual_sym_mode & ALLPERMS, expected_mode.bits());

    let actual_link_mode = lstat(&symlink_path).unwrap().st_mode;
    assert_eq!(link_mode & ALLPERMS, actual_link_mode & ALLPERMS);

    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
    {
        let file = ctx.create(f_type.clone()).unwrap();
        assert!(lchmod(&file, expected_mode).is_ok());
        let actual_mode = stat(&path).unwrap().st_mode;
        assert_eq!(actual_mode & ALLPERMS, expected_mode.bits());
    }
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
crate::test_case! {
    /// chmod successfully change permissions
    change_perm_symlink
}
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
fn change_perm_symlink(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let expected_mode = Mode::from_bits_truncate(0o111);

    assert!(lchmod(&file, expected_mode).is_ok());
    let actual_mode = stat(&file).unwrap().st_mode;
    assert_eq!(actual_mode & ALLPERMS, expected_mode.bits());
}

// chmod/00.t:L58
crate::test_case! {
    /// chmod updates ctime when it succeeds
    update_ctime => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn update_ctime(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    assert_ctime_changed(ctx, &path, || {
        assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_ok());
    });

    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
    assert_ctime_changed(ctx, &path, || {
        assert!(lchmod(&path, Mode::from_bits_truncate(0o111)).is_ok());
    });
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
crate::test_case! {
    /// chmod updates ctime when it succeeds
    update_ctime_symlink
}
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
fn update_ctime_symlink(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Symlink(None)).unwrap();
    assert_ctime_changed(ctx, &path, || {
        assert!(lchmod(&path, Mode::from_bits_truncate(0o111)).is_ok());
    });
}

// chmod/00.t:L89
crate::test_case! {
    /// chmod does not update ctime when it fails
    failed_chmod_unchanged_ctime, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn failed_chmod_unchanged_ctime(ctx: &mut SerializedTestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    let user = ctx.get_new_user();

    assert_ctime_unchanged(ctx, &path, || {
        ctx.as_user(&user, None, || {
            assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_err());
        });
    });
}

crate::test_case! {
    /// chmod does not update ctime when it fails
    failed_chmod_unchanged_ctime_symlink, serialized, root
}
fn failed_chmod_unchanged_ctime_symlink(ctx: &mut SerializedTestContext) {
    let path = ctx.create(FileType::Symlink(None)).unwrap();
    let user = ctx.get_new_user();

    assert_ctime_unchanged(ctx, &path, || {
        ctx.as_user(&user, None, || {
            assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_err());
        });
    });
}

crate::test_case! {
    /// S_ISGID bit shall be cleared upon successful return from chmod of a regular file
    /// if the calling process does not have appropriate privileges, and if
    /// the group ID of the file does not match the effective group ID or one of the
    /// supplementary group IDs
    clear_sgid_bit, serialized, root
}
fn clear_sgid_bit(ctx: &mut SerializedTestContext) {
    let path = ctx.create(FileType::Regular).unwrap();
    chmod(&path, Mode::from_bits_truncate(0o0755)).unwrap();

    let user = ctx.get_new_user();

    chown(&path, Some(user.uid), Some(user.gid)).unwrap();

    let expected_mode = Mode::from_bits_truncate(0o755) | Mode::S_ISGID;
    ctx.as_user(&user, None, || {
        chmod(&path, expected_mode).unwrap();
    });

    let actual_mode = stat(&path).unwrap().st_mode;
    assert_eq!(actual_mode & ALLPERMS_SUID_SGID, expected_mode.bits());

    let expected_mode = Mode::from_bits_truncate(0o755);
    ctx.as_user(&user, None, || {
        chmod(&path, expected_mode).unwrap();
    });

    let actual_mode = stat(&path).unwrap().st_mode;
    assert_eq!(actual_mode & ALLPERMS_SUID_SGID, expected_mode.bits());

    //TODO: FreeBSD "S_ISGID should be removed and chmod(2) should success and FreeBSD returns EPERM."
    #[cfg(not(target_os = "freebsd"))]
    {
        let expected_mode = Mode::from_bits_truncate(0o755) | Mode::S_ISGID;
        ctx.as_user(&user, None, || {
            chmod(&path, expected_mode).unwrap();
        });
        let actual_mode = stat(&path).unwrap().st_mode;
        assert_eq!(actual_mode & ALLPERMS_SUID_SGID, expected_mode.bits());
    }
}

crate::test_case! {
    /// verify SUID/SGID bit behaviour
    verify_suid_sgid, serialized, root
}
fn verify_suid_sgid(ctx: &mut SerializedTestContext) {
    fn check_suid_sgid(ctx: &mut SerializedTestContext, bits: Mode) {
        let mode_bits = Mode::from_bits_truncate(0o777) | bits;
        let mode_without_bits = mode_bits & !bits;
        let user = ctx.get_new_user();
        ctx.as_user(&user, None, || {
            let file = ctx
                .new_file(FileType::Regular)
                .mode(mode_bits.bits())
                .create()
                .unwrap();
            std::fs::write(&file, &[]).unwrap();
            let file_stat = metadata(&file).unwrap();
            assert_eq!(
                file_stat.permissions().mode() as mode_t & ALLPERMS_SUID_SGID,
                mode_without_bits.bits()
            );
        });
    }

    check_suid_sgid(ctx, Mode::S_ISUID);
    check_suid_sgid(ctx, Mode::S_ISGID);
    check_suid_sgid(ctx, Mode::S_ISUID | Mode::S_ISGID);
}
