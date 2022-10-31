use std::{fs::File, io::Write};

use nix::{
    errno::Errno,
    fcntl::{open, posix_fallocate, OFlag},
    sys::stat::{lstat, Mode},
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    test::{FileSystemFeature, TestContext},
    tests::{assert_ctime_changed, assert_ctime_unchanged},
    utils::chmod,
};

crate::test_case! {
    /// posix_fallocate should allocate even if the file is empty
    increase_empty, FileSystemFeature::PosixFallocate
}
fn increase_empty(ctx: &mut TestContext) {
    let expected_size = 567;

    let (path, file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();
    assert!(posix_fallocate(file, 0, expected_size).is_ok());

    let size = lstat(&path).unwrap().st_size;
    assert_eq!(size, expected_size);
}

crate::test_case! {
    /// posix_fallocate should allocate even if the file is not empty
    increase_not_empty, FileSystemFeature::PosixFallocate
}
fn increase_not_empty(ctx: &mut TestContext) {
    let offset = 20_000;
    let size = 3456;

    let (path, file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();
    let mut std_file = File::create(&path).unwrap();
    let random_data: [u8; 1234] = rand::random();
    std_file.write_all(&random_data).unwrap();

    assert!(posix_fallocate(file, offset, size).is_ok());

    let actual_size = lstat(&path).unwrap().st_size;
    assert_eq!(actual_size, offset + size);
}

crate::test_case! {
    /// posix_fallocate should update ctime when it succeeds
    update_ctime_success, FileSystemFeature::PosixFallocate
}
fn update_ctime_success(ctx: &mut TestContext) {
    let (path, file) = ctx.create_file(OFlag::O_RDWR, None).unwrap();

    assert_ctime_changed(ctx, &path, || {
        assert!(posix_fallocate(file, 0, 123).is_ok());
    })
}

crate::test_case! {
    /// posix_fallocate should not update ctime when it fails
    no_update_ctime_fail, FileSystemFeature::PosixFallocate
}
fn no_update_ctime_fail(ctx: &mut TestContext) {
    let (path, file) = ctx.create_file(OFlag::O_WRONLY, None).unwrap();

    assert_ctime_unchanged(ctx, &path, || {
        assert_eq!(posix_fallocate(file, 0, 0), Err(Errno::EINVAL));
    })
}

crate::test_case! {
    /// The file mode of a newly created file should not affect whether
    /// posix_fallocate will work, only the create args
    /// https://bugs.freebsd.org/bugzilla/show_bug.cgi?id=154873
    affected_only_create_flags, serialized, root, FileSystemFeature::PosixFallocate
}
fn affected_only_create_flags(ctx: &mut SerializedTestContext) {
    let subdir = ctx.create(FileType::Dir).unwrap();

    let path = subdir.join("test");
    let file = open(&path, OFlag::O_CREAT | OFlag::O_RDWR, Mode::empty()).unwrap();
    assert!(posix_fallocate(file, 0, 1).is_ok());

    chmod(&subdir, Mode::from_bits_truncate(0o0777)).unwrap();

    let user = ctx.get_new_user();
    ctx.as_user(user, None, || {
        let path = subdir.join("test1");
        let file = open(&path, OFlag::O_CREAT | OFlag::O_RDWR, Mode::empty()).unwrap();
        assert!(posix_fallocate(file, 0, 1).is_ok());
    });
}
