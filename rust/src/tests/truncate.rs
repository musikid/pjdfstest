use std::{fs::File, io::Write};

use nix::{errno::Errno, sys::stat::lstat, unistd::truncate};
use rand::random;

use crate::{
    context::{FileType, SerializedTestContext},
    test::TestContext,
    tests::{assert_ctime_changed, assert_ctime_unchanged},
};

use super::errors::{
    efault::efault_path_test_case,
    eloop::eloop_comp_test_case,
    enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case},
    enoent::{enoent_comp_test_case, enoent_named_file_test_case},
    enotdir::enotdir_comp_test_case,
    erofs::erofs_named_test_case,
    etxtbsy::etxtbsy_test_case,
};

// tests/truncate/00.t

crate::test_case! {
    /// truncate should extend a file, and shrink a sparse file
    extend_file_shrink_sparse
}
fn extend_file_shrink_sparse(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let size = 1234567;
    assert!(truncate(&file, size).is_ok());

    let actual_size = lstat(&file).unwrap().st_size;
    assert_eq!(actual_size, size);

    let size = 567;
    assert!(truncate(&file, size).is_ok());

    let actual_size = lstat(&file).unwrap().st_size;
    assert_eq!(actual_size, size);
}

crate::test_case! {
    /// truncate should shrink the file if the specified size is less than the actual one
    shrink_not_empty
}
fn shrink_not_empty(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let size = 23456;
    let random_data: [u8; 12345] = random();
    File::create(&file)
        .unwrap()
        .write_all(&random_data)
        .unwrap();

    assert!(truncate(&file, size).is_ok());
    let actual_size = lstat(&file).unwrap().st_size;
    assert_eq!(actual_size, size);

    let size = 1;
    assert!(truncate(&file, size).is_ok());
    let actual_size = lstat(&file).unwrap().st_size;
    assert_eq!(actual_size, size);
}

crate::test_case! {
    /// truncate should update ctime if it succeeds
    update_ctime_success
}
fn update_ctime_success(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();

    assert_ctime_changed(ctx, &file, || {
        assert!(truncate(&file, 123).is_ok());
    });
}

crate::test_case! {
    /// truncate should not update ctime if it fails
    unchanged_ctime_failed, serialized, root
}
fn unchanged_ctime_failed(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let user = ctx.get_new_user();

    assert_ctime_unchanged(ctx, &file, || {
        ctx.as_user(user, None, || {
            assert_eq!(truncate(&file, 123), Err(Errno::EACCES));
        });
    });
}

// (f)truncate/01.t
enotdir_comp_test_case!(truncate(~path, 0));

// truncate/02.t
enametoolong_comp_test_case!(truncate(~path, 0));

// truncate/03.t
enametoolong_path_test_case!(truncate(~path, 0));

// (f)truncate/04.t
enoent_named_file_test_case!(truncate(~path, 0));
enoent_comp_test_case!(truncate(~path, 0));

// truncate/07.t
eloop_comp_test_case!(truncate(~path, 0));

crate::test_case! {
    /// truncate returns EISDIR if the named file is a directory
    // truncate/09.t
    eisdir
}
fn eisdir(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Dir).unwrap();
    assert_eq!(truncate(&path, 0), Err(Errno::EISDIR));
}

// (f)truncate/11.t
etxtbsy_test_case!(truncate(~path, 123));

// (f)truncate/12.t
erofs_named_test_case!(truncate(~path, 123));

crate::test_case! {
    /// truncate returns EINVAL if the length argument was less than 0
    // truncate/13.t
    einval_negative_length
}
fn einval_negative_length(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Regular).unwrap();

    assert_eq!(truncate(&path, -1), Err(Errno::EINVAL));
    assert_eq!(truncate(&path, nix::libc::off_t::MIN), Err(Errno::EINVAL));
}

// (f)truncate/14.t
efault_path_test_case!(truncate, |ptr| nix::libc::truncate(ptr, 0));
