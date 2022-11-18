use nix::{sys::stat::fstat, unistd::unlink};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    tests::{assert_ctime_changed, assert_ctime_unchanged},
    utils::link,
};

use super::{
    assert_mtime_changed,
    errors::{
        efault::efault_path_test_case,
        eloop::eloop_comp_test_case,
        enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case},
        enoent::enoent_named_file_test_case,
        enotdir::enotdir_comp_test_case,
    },
};

crate::test_case! {
    /// unlink removes regular, block and char files, symbolic links, fifos and sockets
    // unlink/00.t
    remove_type => [Regular, Block, Char, Symlink(None), Fifo, Socket]
}
fn remove_type(ctx: &mut TestContext, ft: FileType) {
    let path = ctx.create(ft).unwrap();

    assert!(unlink(&path).is_ok());
    assert!(!path.exists());
}

crate::test_case! {
    /// successful unlink(2) updates ctime.
    // unlink/00.t
    update_ctime_success => [Regular, Block, Char, Fifo, Socket]
}
fn update_ctime_success(ctx: &mut TestContext, ft: FileType) {
    let path = ctx.create(ft).unwrap();

    let link_path = ctx.base_path().join("link");
    link(&path, &link_path).unwrap();

    assert_ctime_changed(ctx, &link_path, || {
        assert!(unlink(&path).is_ok());
    });
}

// TODO: why it isn't in the original test suite?
// crate::test_case! {
//     /// successful unlink(2) updates ctime (symlink).
//     update_ctime_success_symlink
// }
// fn update_ctime_success_symlink(ctx: &mut TestContext) {
//     let path = ctx.create(FileType::Symlink(None)).unwrap();
//     let link_path = ctx.base_path().join("link");
//     link(&path, &link_path).unwrap();
//     assert_ctime_changed(ctx, &link_path, || {
//         assert!(unlink(&path).is_ok());
//     });
// }

crate::test_case! {
    /// unsuccessful unlink(2) does not update ctime.
    // unlink/00.t
    unchanged_ctime_failed, serialized, root => [Regular, Block, Char, Fifo, Socket]
}
fn unchanged_ctime_failed(ctx: &mut SerializedTestContext, ft: FileType) {
    let path = ctx.create(ft).unwrap();

    let link_path = ctx.base_path().join("link");
    link(&path, &link_path).unwrap();

    let user = ctx.get_new_user();

    ctx.as_user(&user, None, || {
        assert_ctime_unchanged(ctx, &link_path, || {
            assert!(unlink(&path).is_err());
        });
    });
}

// TODO: why it isn't in the original test suite?
// crate::test_case! {
//     /// unsuccessful unlink(2) does not update ctime.
//     unchanged_ctime_failed_symlink, serialized, root => [Regular, Fifo, Socket]
// }
// fn unchanged_ctime_failed_symlink(ctx: &mut SerializedTestContext, ft: FileType) {
//     let path = ctx.create(ft).unwrap();

//     let link_path = ctx.base_path().join("link");
//     link(&path, &link_path).unwrap();

//     let user = User::from_uid(Uid::from_raw(65534)).unwrap().unwrap();

//     ctx.as_user(Some(user.uid), Some(user.gid), || {
//         assert_ctime_unchanged(ctx, &link_path, || {
//             assert!(unlink(&path).is_err());
//         });
//     });
// }

crate::test_case! {
    /// successful unlink(2) on a directory entry updates ctime and mtime for the parent folder.
    // unlink/00.t
    update_mtime_ctime_success_folder => [Regular, Block, Char, Fifo, Socket, Symlink(None)]
}
fn update_mtime_ctime_success_folder(ctx: &mut TestContext, ft: FileType) {
    let dir = ctx.new_file(FileType::Dir).create().unwrap();
    let file = ctx.new_file(ft).name(dir.join("file")).create().unwrap();

    assert_mtime_changed(ctx, &dir, || {
        assert_ctime_changed(ctx, &dir, || {
            assert!(unlink(&file).is_ok());
        });
    })
}

crate::test_case! {
    /// An open file will not be immediately freed by unlink
    // unlink/14.t
    open_file_not_freed
}
fn open_file_not_freed(ctx: &mut TestContext) {
    let (path, file) = ctx
        .create_file(nix::fcntl::OFlag::O_RDWR, Some(0o644))
        .unwrap();

    assert!(unlink(&path).is_ok());

    let fd_stat = fstat(file).unwrap();
    // A deleted file's link count should be 0
    assert_eq!(fd_stat.st_nlink, 0);

    // I/O to open but deleted files should work, too
    const EXAMPLE_BYTES: &str = "Hello, World!";
    nix::unistd::write(file, EXAMPLE_BYTES.as_bytes()).unwrap();
    let mut buf = [0; EXAMPLE_BYTES.len()];
    nix::sys::uio::pread(file, &mut buf, 0).unwrap();
    assert_eq!(buf, EXAMPLE_BYTES.as_bytes());
}

// unlink/01.t
enotdir_comp_test_case!(unlink);

// unlink/02.t
enametoolong_comp_test_case!(unlink);

// unlink/03.t
enametoolong_path_test_case!(unlink);

// unlink/04.t
enoent_named_file_test_case!(unlink);

// unlink/07.t
eloop_comp_test_case!(unlink);

// unlink/13.t
efault_path_test_case!(unlink, nix::libc::unlink);
