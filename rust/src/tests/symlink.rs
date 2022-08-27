use std::{
    fs::{metadata, remove_dir, remove_file, symlink_metadata},
    os::unix::{fs::symlink, prelude::FileTypeExt},
    path::Path,
};

use nix::{errno::Errno, sys::stat::stat};

use crate::runner::context::{FileType, TestContext};

use super::{assert_ctime_changed, assert_mtime_changed};

crate::test_case! {
    /// symlink creates symbolic links
    // symlink/00.t
    create_symlink => [Regular, Dir, Block, Char, Fifo]
}
fn create_symlink(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();
    let link = ctx.gen_path();
    symlink(&file, &link).unwrap();

    let link_stat = symlink_metadata(&link).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    let follow_link_type = follow_link_stat.file_type();
    assert!(link_stat.is_symlink());
    assert!(match ft {
        FileType::Regular => follow_link_type.is_file(),
        FileType::Dir => follow_link_type.is_dir(),
        FileType::Block => follow_link_type.is_block_device(),
        FileType::Char => follow_link_type.is_char_device(),
        FileType::Fifo => follow_link_type.is_fifo(),
        _ => unreachable!(),
    });

    match ft {
        FileType::Dir => remove_dir(&file),
        _ => remove_file(&file),
    }
    .unwrap();
    assert_eq!(stat(&link).unwrap_err(), Errno::ENOENT);
}

crate::test_case! {
    /// symlink create a symbolic link to a symbolic link
    // symlink/00.t
    create_symlink_to_symlink
}
fn create_symlink_to_symlink(ctx: &mut TestContext) {
    let target = ctx.create(FileType::Regular).unwrap();
    let file = ctx.create(FileType::Symlink(Some(target))).unwrap();
    let link = ctx.gen_path();
    symlink(&file, &link).unwrap();

    let link_stat = symlink_metadata(&link).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    let follow_link_type = follow_link_stat.file_type();
    assert!(link_stat.is_symlink());
    assert!(follow_link_type.is_file());

    remove_file(&file).unwrap();
    assert_eq!(stat(&link).unwrap_err(), Errno::ENOENT);
}

crate::test_case! {
    /// symlink should update parent's ctime and mtime on success
    changed_parent_time_success
}
fn changed_parent_time_success(ctx: &mut TestContext) {
    // TODO: Migrate to new time asssertion syntax when merged
    assert_ctime_changed(ctx, ctx.base_path(), || {
        assert_mtime_changed(ctx, ctx.base_path(), || {
            ctx.create(FileType::Symlink(None)).unwrap();
        })
    });
}

// symlink/8.t
crate::eexist_test_case! {symlink, |_ctx, path|
    crate::utils::symlink(Path::new("nonexistent"), path)
}
