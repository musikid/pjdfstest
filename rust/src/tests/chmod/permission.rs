use std::{thread::sleep, time::Duration};

use crate::{
    pjdfs_test_case,
    runner::context::FileType,
    test::{TestContext, TestResult},
    test_assert, test_assert_eq,
    tests::chmod::chmod,
};
use nix::{
    libc::mode_t,
    sys::stat::{lstat, stat, Mode},
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
fn test_change_perm(ctx: &mut TestContext) -> TestResult {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type)?;
        let expected_mode = Mode::from_bits_truncate(0o111);

        chmod(&path, expected_mode)?;

        let actual_mode = stat(&path)?.st_mode;

        test_assert_eq!(actual_mode & FILE_PERMS, expected_mode.bits());

        // We test if it applies through symlinks
        let symlink_path = ctx.create(FileType::Symlink(Some(path.clone())))?;
        let link_mode = lstat(&symlink_path)?.st_mode;
        let expected_mode = Mode::from_bits_truncate(0o222);

        chmod(&symlink_path, expected_mode)?;

        let actual_mode = stat(&path)?.st_mode;
        let actual_sym_mode = stat(&symlink_path)?.st_mode;
        test_assert_eq!(actual_mode & FILE_PERMS, expected_mode.bits());
        test_assert_eq!(actual_sym_mode & FILE_PERMS, expected_mode.bits());

        let actual_link_mode = lstat(&symlink_path)?.st_mode;
        test_assert_eq!(link_mode & FILE_PERMS, actual_link_mode & FILE_PERMS);
    }

    Ok(())
}

// chmod/00.t:L58
fn test_ctime(ctx: &mut TestContext) -> TestResult {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type)?;
        let ctime_before = stat(&path)?.st_ctime;

        sleep(Duration::from_secs(1));

        chmod(&path, Mode::from_bits_truncate(0o111))?;

        let ctime_after = stat(&path)?.st_ctime;
        test_assert!(ctime_after > ctime_before);
    }

    Ok(())
}

// chmod/00.t:L89
fn test_failed_chmod_unchanged_ctime(ctx: &mut TestContext) -> TestResult {
    for f_type in FileType::iter().filter(|ft| *ft != FileType::Symlink(None)) {
        let path = ctx.create(f_type)?;
        let ctime_before = stat(&path)?.st_ctime;

        sleep(Duration::from_secs(1));

        ctx.as_user(Some(Uid::from_raw(65534)), None, || {
            test_assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_err());
            Ok(())
        })?;

        let ctime_after = stat(&path)?.st_ctime;
        test_assert_eq!(ctime_after, ctime_before);
    }

    Ok(())
}

fn test_clear_isgid_bit(ctx: &mut TestContext) -> TestResult {
    let path = ctx.create(FileType::Regular)?;
    chmod(&path, Mode::from_bits_truncate(0o0755))?;

    let user = Uid::from_raw(65535);
    let group = Gid::from_raw(65535);

    chown(&path, Some(user), Some(group))?;

    let expected_mode = Mode::from_bits_truncate(0o2755);
    ctx.as_user(Some(user), Some(group), || {
        chmod(&path, expected_mode)?;
        Ok(())
    })?;

    let actual_mode = stat(&path)?.st_mode;
    test_assert_eq!(actual_mode & 0o7777, expected_mode.bits());

    let expected_mode = Mode::from_bits_truncate(0o0755);
    ctx.as_user(Some(user), Some(group), || {
        chmod(&path, expected_mode)?;
        Ok(())
    })?;

    let actual_mode = stat(&path)?.st_mode;
    test_assert_eq!(actual_mode & 0o7777, expected_mode.bits());
    //TODO: FreeBSD "S_ISGID should be removed and chmod(2) should success and FreeBSD returns EPERM."

    Ok(())
}
