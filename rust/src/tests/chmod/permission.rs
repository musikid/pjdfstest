use std::{thread::sleep, time::Duration};

use crate::{pjdfs_test_case, runner::context::FileType, test::TestContext, tests::chmod::chmod};
use nix::{
    sys::stat::{lstat, mode_t, stat, Mode},
    unistd::{chown, Gid, Uid},
};
use strum::IntoEnumIterator;

pjdfs_test_case!(
    permission,
    { test: test_ctime, require_root: true },
    { test: test_change_perm, require_root: true },
    { test: test_failed_chmod_unchanged_ctime, require_root: true },
    { test: test_clear_isgid_bit }
);

const FILE_PERMS: mode_t = 0o777;

// chmod/00.t:L24
fn test_change_perm(ctx: &mut TestContext) {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type).unwrap();
        let expected_mode = Mode::from_bits_truncate(0o111);

        chmod(&path, expected_mode).unwrap();

        let actual_mode = stat(&path).unwrap().st_mode;

        assert_eq!(actual_mode & FILE_PERMS, expected_mode.bits());

        // We test if it applies through symlinks
        let symlink_path = ctx.create(FileType::Symlink(Some(path.clone()))).unwrap();
        let link_mode = lstat(&symlink_path).unwrap().st_mode;
        let expected_mode = Mode::from_bits_truncate(0o222);

        chmod(&symlink_path, expected_mode).unwrap();

        let actual_mode = stat(&path).unwrap().st_mode;
        let actual_sym_mode = stat(&symlink_path).unwrap().st_mode;
        assert_eq!(actual_mode & FILE_PERMS, expected_mode.bits());
        assert_eq!(actual_sym_mode & FILE_PERMS, expected_mode.bits());

        let actual_link_mode = lstat(&symlink_path).unwrap().st_mode;
        assert_eq!(link_mode & FILE_PERMS, actual_link_mode & FILE_PERMS);
    }
}

// chmod/00.t:L58
fn test_ctime(ctx: &mut TestContext) {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type).unwrap();
        let ctime_before = stat(&path).unwrap().st_ctime;

        sleep(Duration::from_secs(1));

        chmod(&path, Mode::from_bits_truncate(0o111)).unwrap();

        let ctime_after = stat(&path).unwrap().st_ctime;
        assert!(ctime_after > ctime_before);
    }
}

// chmod/00.t:L89
fn test_failed_chmod_unchanged_ctime(ctx: &mut TestContext) {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type).unwrap();
        let ctime_before = stat(&path).unwrap().st_ctime;

        sleep(Duration::from_secs(1));

        ctx.as_user(Some(Uid::from_raw(65534)), None, || {
            assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_err());
        });

        let ctime_after = stat(&path).unwrap().st_ctime;
        assert_eq!(ctime_after, ctime_before);
    }
}

fn test_clear_isgid_bit(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Regular).unwrap();
    chmod(&path, Mode::from_bits_truncate(0o0755)).unwrap();

    let user = Uid::from_raw(65535);
    let group = Gid::from_raw(65535);

    chown(&path, Some(user), Some(group)).unwrap();

    let expected_mode = Mode::from_bits_truncate(0o2755);
    ctx.as_user(Some(user), Some(group), || {
        chmod(&path, expected_mode).unwrap();
    });

    let actual_mode = stat(&path).unwrap().st_mode;
    assert_eq!(actual_mode & 0o7777, expected_mode.bits());

    let expected_mode = Mode::from_bits_truncate(0o0755);
    ctx.as_user(Some(user), Some(group), || {
        chmod(&path, expected_mode).unwrap();
    });

    let actual_mode = stat(&path).unwrap().st_mode;
    assert_eq!(actual_mode & 0o7777, expected_mode.bits());
    //TODO: FreeBSD "S_ISGID should be removed and chmod(2) should success and FreeBSD returns EPERM."
}
