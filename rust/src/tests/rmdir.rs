use std::fs::metadata;

use cfg_if::cfg_if;
use nix::{errno::Errno, sys::stat::lstat};

use crate::{
    runner::context::{FileBuilder, FileType, TestContext},
    tests::assert_mtime_changed,
    utils::rmdir,
};

use super::assert_ctime_changed;

crate::test_case! {
    /// rmdir remove directory
    remove_dir
}
fn remove_dir(ctx: &mut TestContext) {
    let dir = ctx.create(crate::runner::context::FileType::Dir).unwrap();
    assert!(metadata(&dir).unwrap().is_dir());
    assert!(rmdir(&dir).is_ok());
    assert_eq!(lstat(&dir).unwrap_err(), Errno::ENOENT);
}

crate::test_case! {
    /// rmdir updates parent ctime and mtime on success
    changed_time_parent_success
}
fn changed_time_parent_success(ctx: &mut TestContext) {
    let dir = ctx.create(crate::runner::context::FileType::Dir).unwrap();
    assert_ctime_changed(ctx, ctx.base_path(), || {
        assert_mtime_changed(ctx, ctx.base_path(), || {
            assert!(rmdir(&dir).is_ok());
        });
    });
}

crate::test_case! {
    /// rmdir returns EEXIST or ENOTEMPTY if the named directory contains files
    /// other than '.' and '..' in it
    enotempty => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
    // rmdir/06.t
}
fn enotempty(ctx: &mut TestContext, ft: FileType) {
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let _file0 = FileBuilder::new(ft, &dir0).create().unwrap();
    let e = rmdir(&dir0).unwrap_err();
    assert!(e == Errno::EEXIST || e == Errno::ENOTEMPTY);
}

crate::test_case! {
    /// rmdir returns EINVAL if the last component of the path is '.'
    einval_dot
    //rmdir/12.t
}
fn einval_dot(ctx: &mut TestContext) {
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let dir1 = FileBuilder::new(FileType::Dir, &dir0).create().unwrap();
    assert_eq!(Err(Errno::EINVAL), rmdir(&dir1.join(".")));
}

crate::test_case! {
    /// rmdir returns EEXIST or ENOTEMPTY if the last component of the path is
    /// '..'
    enotempty_dotdot
    //rmdir/12.t
}
fn enotempty_dotdot(ctx: &mut TestContext) {
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let dir1 = FileBuilder::new(FileType::Dir, &dir0).create().unwrap();
    let e = rmdir(&dir1.join("..")).unwrap_err();
    cfg_if! {
        if #[cfg(target_os = "freebsd")] {
            // XXX FreeBSD's behavior here is not POSIX compliant
            assert_eq!(e, Errno::EINVAL);
        } else {
            assert!(e == Errno::EEXIST || e == Errno::ENOTEMPTY, "{:?}", e);
        }
    }
}
