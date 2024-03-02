//! Tests for ACL_APPEND_DATA
use nix::{errno::Errno, unistd::unlink};

use super::prependacl;
use crate::{
    context::{FileBuilder, FileType, SerializedTestContext},
    test::FileSystemFeature,
    utils::{rename, rmdir},
};

crate::test_case! {
    /// ACL_APPEND_DATA on a directory allows a user to create directories
    // granular/00.t:L87
    can_create_directories, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn can_create_directories(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let path = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();

    prependacl(&path, &format!("allow::user:{}:append", user.gid));

    ctx.as_user(user, None, move || {
        FileBuilder::new(FileType::Dir, &path).create().unwrap();
    });
}

crate::test_case! {
    /// ACL_APPEND_DATA on a directory does not allow a user to create files
    // granular/00.t:L78
    cant_create_files, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Symlink(None)]
}
fn cant_create_files(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let path = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();

    prependacl(&path, &format!("allow::user:{}:append", user.gid));

    ctx.as_user(user, None, move || {
        let e = FileBuilder::new(ft, &path).create().unwrap_err();
        assert_eq!(Errno::EACCES, e);
    });
}

crate::test_case! {
    /// ACL_APPEND_DATA on a directory does not allow a user to move in files
    /// from elsewhere.
    // granular/00.t:L92
    cant_rename_files, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn cant_rename_files(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let odir = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let oldpath = FileBuilder::new(FileType::Regular, &odir).create().unwrap();
    let newpath = dir.join("new");

    prependacl(&dir, &format!("allow::user:{}:append", user.uid));

    ctx.as_user(user, None, move || {
        let e = rename(&oldpath, &newpath).unwrap_err();
        assert_eq!(Errno::EACCES, e);
    });
}

crate::test_case! {
    /// ACL_APPEND_DATA on a directory allows a user to move in
    /// directories from elsewhere, overwriting existing ones if necessary.
    // granular/00.t:L100
    can_rename_directories, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn can_rename_directories(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let odir = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let oldpath = FileBuilder::new(FileType::Dir, &odir)
        .mode(0o777)
        .create()
        .unwrap();
    //let newpath = FileBuilder::new(FileType::Dir, &dir).create().unwrap();
    let newpath = dir.join("new");

    prependacl(&dir, &format!("allow::user:{}:append", user.uid));

    ctx.as_user(user, None, move || {
        rename(&oldpath, &newpath).unwrap();
    });
}

crate::test_case! {
    /// ACL_APPEND_DATA on a directory does not allow a user to remove other
    /// users' directories.
    // granular/00.t:L53
    rmdir_err, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn rmdir_err(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let path = FileBuilder::new(FileType::Dir, &dir).create().unwrap();

    prependacl(&dir, &format!("allow::user:{}:append", user.uid));

    ctx.as_user(user, None, move || {
        assert_eq!(Err(Errno::EACCES), rmdir(&path));
    });
}

crate::test_case! {
    /// ACL_WRITE_DATA on a directory does not allow a user to remove
    /// files.
    // granular/00.t:L89
    unlink_err, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Symlink(None)]
}
fn unlink_err(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let path = FileBuilder::new(ft, &dir).create().unwrap();

    prependacl(&dir, &format!("allow::user:{}:append", user.uid));

    ctx.as_user(user, None, move || {
        assert_eq!(Err(Errno::EACCES), unlink(&path));
    });
}
