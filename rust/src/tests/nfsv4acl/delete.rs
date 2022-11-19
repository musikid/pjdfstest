//! Tests for ACL_WRITE_DATA
use nix::unistd::unlink;

use super::prependacl;
use crate::{
    context::{FileBuilder, FileType, SerializedTestContext},
    test::FileSystemFeature,
    utils::{rename, rmdir},
};

crate::test_case! {
    /// Denied DELETE does not prohibit rmdir, if the user has WRITE_DATA on
    /// the directory.
    // granular/03.t:L51
    // IMHO this is a bug in ZFS.  NFSv4 ACLs would be more useful if WRITE_DATA
    // on a directory did not grant permission to to delete children.  RFC 3530
    // is not clear on the matter.
    denied_delete_does_not_prohibit_rmdir, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn denied_delete_does_not_prohibit_rmdir(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let file = FileBuilder::new(FileType::Dir, &dir).create().unwrap();

    prependacl(&dir, &format!("allow::user:{}:write_data", user.uid));
    prependacl(&file, &format!("deny::user:{}:delete", user.uid));

    ctx.as_user(&user, None, move || {
        rmdir(&file).unwrap();
    });
}

crate::test_case! {
    /// Denied DELETE does not prohibit unlink, if the user has WRITE_DATA on
    /// the directory.
    // granular/03.t:L51
    // IMHO this is a bug in ZFS.  NFSv4 ACLs would be more useful if WRITE_DATA
    // on a directory did not grant permission to to delete children.  RFC 3530
    // is not clear on the matter.
    denied_delete_does_not_prohibit_unlink, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn denied_delete_does_not_prohibit_unlink(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let dir = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let file = FileBuilder::new(FileType::Regular, &dir).create().unwrap();

    prependacl(&dir, &format!("allow::user:{}:write_data", user.uid));
    prependacl(&file, &format!("deny::user:{}:delete", user.uid));

    ctx.as_user(&user, None, move || {
        unlink(&file).unwrap();
    });
}

crate::test_case! {
    /// Denied DELETE does not prohibit rename, if the user has WRITE_DATA on
    /// the directory.
    // granular/03.t:L56
    denied_delete_does_not_prohibit_rename, serialized, root,
        FileSystemFeature::Nfsv4Acls => [Regular, Dir]
}
fn denied_delete_does_not_prohibit_rename(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let dir1 = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let file = FileBuilder::new(ft.clone(), &dir0)
        .mode(0o777)
        .create()
        .unwrap();
    let newpath = dir1.join("new");

    prependacl(&dir0, &format!("allow::user:{}:write_data", user.uid));
    prependacl(&file, &format!("deny::user:{}:delete", user.uid));
    if ft == FileType::Dir {
        // IMHO it's a bug in ZFS that WRITE_DATA is sufficient to delete
        // directory entries, but APPEND_DATA is required to create them.
        prependacl(&dir0, &format!("allow::user:{}:append", user.uid));
    }

    ctx.as_user(&user, None, move || {
        rename(&file, &newpath).unwrap();
        rename(&newpath, &file).unwrap();
    });
}

crate::test_case! {
    /// DELETE allows for unlinking directories, no matter what the permissions
    /// on the parent directory are.
    // granular/05.t:L87
    delete_rmdir, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn delete_rmdir(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let path = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();

    prependacl(&path, &format!("allow::user:{}:delete", user.uid));

    ctx.as_user(&user, None, move || {
        rmdir(&path).unwrap();
    });
}

crate::test_case! {
    /// DELETE allows for unlinking, no matter what the permissions on the
    /// parent directory are.
    // granular/03.t:L83
    delete_unlink, serialized, root, FileSystemFeature::Nfsv4Acls
}
fn delete_unlink(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    let path = ctx.new_file(FileType::Regular).create().unwrap();

    prependacl(&path, &format!("allow::user:{}:delete", user.uid));

    ctx.as_user(&user, None, move || {
        unlink(&path).unwrap();
    });
}

crate::test_case! {
    /// DELETE allows for moving out of a directory, no matter what the
    /// permissions on the parent directory are.
    // granular/03.t:L88
    // granular/05.t:L92
    delete_rename, serialized, root, FileSystemFeature::Nfsv4Acls
        => [Regular, Dir]
}
fn delete_rename(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let dir0 = ctx.new_file(FileType::Dir).mode(0o755).create().unwrap();
    let dir1 = ctx.new_file(FileType::Dir).mode(0o777).create().unwrap();
    let file = FileBuilder::new(ft, &dir0).mode(0o777).create().unwrap();
    let newpath = dir1.join("new");

    prependacl(&file, &format!("allow::user:{}:delete", user.uid));

    ctx.as_user(&user, None, move || {
        rename(&file, &newpath).unwrap();
    });
}
