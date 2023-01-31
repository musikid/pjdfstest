use std::{
    fs::{metadata, remove_dir, remove_file, symlink_metadata},
    os::unix::prelude::FileTypeExt,
    path::Path,
};

use crate::{
    context::{FileType, TestContext},
    tests::{assert_times_changed, errors::enoent::enoent_comp_test_case, CTIME, MTIME},
    utils::symlink,
};

use super::errors::{
    efault::efault_either_test_case,
    enametoolong::{enametoolong_comp_test_case, enametoolong_either_path_test_case},
    enotdir::enotdir_comp_test_case,
    erofs::erofs_new_file_test_case,
};

crate::test_case! {
    /// symlink creates symbolic links
    // symlink/00.t
    create_symlink => [Regular, Dir, Block, Char, Fifo]
}
fn create_symlink(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();
    let link = ctx.gen_path();
    assert!(symlink(&file, &link).is_ok());

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
    assert!(!link.exists());
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
    assert!(symlink(&file, &link).is_ok());

    let link_stat = symlink_metadata(&link).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    let follow_link_type = follow_link_stat.file_type();
    assert!(link_stat.is_symlink());
    assert!(follow_link_type.is_file());

    remove_file(&file).unwrap();
    assert!(!link.exists());
}

crate::test_case! {
    /// symlink should update parent's ctime and mtime on success
    changed_parent_time_success
}
fn changed_parent_time_success(ctx: &mut TestContext) {
    assert_times_changed()
        .path(ctx.base_path(), CTIME | MTIME)
        .execute(ctx, true, || {
            let link = ctx.gen_path();

            assert!(symlink(Path::new("test"), &link).is_ok());
        });
}

// symlink/01.t
enotdir_comp_test_case!(symlink(Path::new("test"), ~path));

// symlink/02.t
// TODO: Clarify that only path2 is checked
enametoolong_comp_test_case!(symlink(Path::new("test"), ~path));

// symlink/03.t
enametoolong_either_path_test_case!(symlink);

// symlink/04.t
enoent_comp_test_case!(symlink(Path::new("test"), ~path));

// symlink/10.t
erofs_new_file_test_case!(symlink(Path::new("test"), ~path));

// symlink/13.t
efault_either_test_case!(symlink, nix::libc::symlink);
