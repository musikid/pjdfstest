use std::{fs::File, io::Write};

use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::{lstat, Mode},
    unistd::ftruncate,
};
use rand::random;

use crate::{
    runner::context::FileType,
    test::{SerializedTestContext, TestContext},
    tests::{assert_ctime_changed, assert_ctime_unchanged},
    utils::chmod,
};

// tests/ftruncate/00.t

crate::test_case! {
    /// ftruncate extends if the file is empty
    increase_empty
}
fn increase_empty(ctx: &mut TestContext) {
    let (path, file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();
    let size = 1234567;
    ftruncate(file, size).unwrap();

    let actual_size = lstat(&path).unwrap().st_size;
    assert_eq!(actual_size, size);

    let wronly_file = open(&path, OFlag::O_WRONLY, Mode::empty()).unwrap();
    let size = 567;
    ftruncate(wronly_file, size).unwrap();

    let actual_size = lstat(&path).unwrap().st_size;
    assert_eq!(actual_size, size);
}

crate::test_case! {
    /// ftruncate should shrink the file if the specified size is less than the actual one
    truncate_not_empty
}
fn truncate_not_empty(ctx: &mut TestContext) {
    let (path, file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();
    let size = 23456;
    let random_data: [u8; 12345] = random();
    File::create(&path)
        .unwrap()
        .write_all(&random_data)
        .unwrap();

    ftruncate(file, size).unwrap();
    let actual_size = lstat(&path).unwrap().st_size;
    assert_eq!(actual_size, size);

    let wronly_file = open(&path, OFlag::O_WRONLY, Mode::empty()).unwrap();

    let size = 1;
    ftruncate(wronly_file, size).unwrap();
    let actual_size = lstat(&path).unwrap().st_size;
    assert_eq!(actual_size, size);
}

crate::test_case! {
    /// ftruncate should update ctime if it succeeds
    update_ctime_success
}
fn update_ctime_success(ctx: &mut TestContext) {
    let (path, file) = ctx.create_file(OFlag::O_RDWR, Some(0o644)).unwrap();

    assert_ctime_changed(ctx, &path, || {
        ftruncate(file, 123).unwrap();
    });
}
crate::test_case! {
    /// ftruncate should not update ctime if it fails
    unchanged_ctime_failed
}
fn unchanged_ctime_failed(ctx: &mut TestContext) {
    let (path, file) = ctx.create_file(OFlag::O_RDONLY, Some(0o644)).unwrap();

    assert_ctime_unchanged(ctx, &path, || {
        assert_eq!(ftruncate(file, 123).unwrap_err(), Errno::EINVAL);
    });
}

crate::test_case! {
    /// The file mode of a newly created file should not affect whether ftruncate
    /// will work, only the create args
    /// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=154873
    affected_create_flags_only, serialized, root
}
fn affected_create_flags_only(ctx: &mut SerializedTestContext) {
    let subdir = ctx.create(FileType::Dir).unwrap();
    let path = subdir.join("test");

    let file = open(&path, OFlag::O_CREAT | OFlag::O_RDWR, Mode::empty()).unwrap();
    assert!(ftruncate(file, 0).is_ok());

    chmod(&subdir, Mode::from_bits_truncate(0o777)).unwrap();

    let path = subdir.join("unprivileged");

    let user = ctx.get_new_user();

    ctx.as_user(&user, None, || {
        let file = open(&path, OFlag::O_CREAT | OFlag::O_RDWR, Mode::empty()).unwrap();
        assert!(ftruncate(file, 0).is_ok());
    });
}
