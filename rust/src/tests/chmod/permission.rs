use std::{thread::sleep, time::Duration};

use crate::{runner::context::FileType, test::TestContext, tests::chmod::chmod};
use nix::{
    sys::stat::{lstat, mode_t, stat, Mode},
    unistd::{chown, Gid, Uid},
};
use strum::IntoEnumIterator;

const FILE_PERMS: mode_t = 0o777;

// chmod/00.t:L24
crate::test_case! {change_perm => [FileType::Regular, FileType::Dir, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn change_perm(ctx: &mut TestContext, f_type: FileType) {
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

// chmod/00.t:L58
crate::test_case! {ctime => [FileType::Regular, FileType::Dir, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn ctime(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    let ctime_before = stat(&path).unwrap().st_ctime;

    sleep(Duration::from_secs(1));

    chmod(&path, Mode::from_bits_truncate(0o111)).unwrap();

    let ctime_after = stat(&path).unwrap().st_ctime;
    assert!(ctime_after > ctime_before);
}

// chmod/00.t:L89
crate::test_case! {failed_chmod_unchanged_ctime => [FileType::Regular, FileType::Dir, FileType::Fifo, FileType::Block, FileType::Char, FileType::Socket]}
fn failed_chmod_unchanged_ctime(ctx: &mut TestContext, f_type: FileType) {
    let path = ctx.create(f_type).unwrap();
    let ctime_before = stat(&path).unwrap().st_ctime;

    sleep(Duration::from_secs(1));

    ctx.as_user(Some(Uid::from_raw(65534)), None, || {
        assert!(chmod(&path, Mode::from_bits_truncate(0o111)).is_err());
    });

    let ctime_after = stat(&path).unwrap().st_ctime;
    assert_eq!(ctime_after, ctime_before);
}

crate::test_case! {clear_isgid_bit}
fn clear_isgid_bit(ctx: &mut TestContext) {
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
