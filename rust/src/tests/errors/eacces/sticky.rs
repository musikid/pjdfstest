use std::path::Path;

use nix::{
    errno::Errno,
    sys::stat::{lstat, Mode},
    unistd::{chown, unlink, Uid, User},
};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    tests::AsTimeInvariant,
    utils::{chmod, lchown, rename, rmdir, ALLPERMS},
};

macro_rules! sticky {
    ($ctx: ident, $syscall: ident, $file_builder: expr, $check: expr) => {
        let user = $ctx.get_new_user();
        let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
        let other_user = $ctx.get_new_user();
        let file_builder = $file_builder;
        let success_checker = $check;
        let different_users = &[current_user, other_user];

        // User owns both: the sticky directory and the file.
        chmod(
            $ctx.base_path(),
            Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX,
        )
        .unwrap();
        chown($ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
        let file = file_builder($ctx);

        $ctx.as_user(&user, None, || {
            assert!($syscall(&file).is_ok());
        });
        assert!(success_checker(&file));

        // User owns the sticky directory, but doesn't own the file.
        chown($ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
        for other_user in different_users {
            let file = file_builder($ctx);
            lchown(&file, Some(other_user.uid), Some(other_user.gid)).unwrap();

            $ctx.as_user(&user, None, || {
                assert!($syscall(&file).is_ok());
            });
            assert!(success_checker(&file));
        }

        // User owns the file, but doesn't own the sticky directory.
        for other_user in different_users {
            chown($ctx.base_path(), Some(other_user.uid), Some(other_user.gid)).unwrap();
            let file = file_builder($ctx);
            lchown(&file, Some(user.uid), Some(user.gid)).unwrap();

            $ctx.as_user(&user, None, || {
                assert!($syscall(&file).is_ok());
            });
            assert!(success_checker(&file));
        }

        // User doesn't own the sticky directory nor the file.
        for other_user in different_users {
            chown($ctx.base_path(), Some(other_user.uid), Some(other_user.gid)).unwrap();
            let file = file_builder($ctx);
            lchown(&file, Some(other_user.uid), Some(other_user.gid)).unwrap();

            $ctx.as_user(&user, None, || {
                assert!(matches!($syscall(&file), Err(Errno::EACCES | Errno::EPERM)));
            });
        }
    };

    (rename, $ctx: ident, $file_builder: expr, $check: expr) => {};
}

crate::test_case! {
    /// unlink returns EACCES or EPERM if the directory containing the file is marked sticky, and neither the containing directory
    /// nor the file to be removed are owned by the effective user ID
    // unlink/11.t
    unlink_file_sticky_dir_file_not_euid, serialized, root => [Regular, Fifo, Block, Char, Socket, Symlink(None)]
}
fn unlink_file_sticky_dir_file_not_euid(ctx: &mut SerializedTestContext, ft: FileType) {
    sticky!(
        ctx,
        unlink,
        |ctx: &mut TestContext| ctx.create(ft.clone()).unwrap(),
        |path: &Path| { !path.exists() }
    );
}

crate::test_case! {
    /// rmdir returns EACCES or EPERM if the directory containing the file is marked sticky, and neither the containing directory
    /// nor the file to be removed are owned by the effective user ID
    // rmdir/11.t
    rmdir_file_sticky_dir_file_not_euid, serialized, root
}
fn rmdir_file_sticky_dir_file_not_euid(ctx: &mut SerializedTestContext) {
    sticky!(
        ctx,
        rmdir,
        |ctx: &mut TestContext| ctx.create(FileType::Dir).unwrap(),
        |path: &Path| { !path.exists() }
    );
}
