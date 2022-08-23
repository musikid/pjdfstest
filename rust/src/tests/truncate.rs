use std::{fs::File, io::Write};

use nix::{errno::Errno, sys::stat::lstat, unistd::truncate};
use rand::random;

use crate::{
    runner::context::{FileType, SerializedTestContext},
    test::TestContext,
    tests::{assert_ctime_changed, assert_ctime_unchanged},
};

// tests/truncate/00.t

crate::test_case! {
    /// truncate should shrink a sparse file
    shrink_sparse
}
fn shrink_sparse(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let size = 1234567;
    truncate(&file, size).unwrap();

    let actual_size = lstat(&file).unwrap().st_size;
    assert_eq!(actual_size, size);

    let size = 567;
    truncate(&file, size).unwrap();

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

    truncate(&file, size).unwrap();
    let actual_size = lstat(&file).unwrap().st_size;
    assert_eq!(actual_size, size);

    let size = 1;
    truncate(&file, size).unwrap();
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
        truncate(&file, 123).unwrap();
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
        ctx.as_user(&user, None, || {
            assert_eq!(truncate(&file, 123), Err(Errno::EACCES));
        });
    });
}
