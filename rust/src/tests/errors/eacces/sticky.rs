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

/// We need to do a cartesian product between the `from` and the `to` file types.
/// Hopefully, this shouldn't grow anymore.
macro_rules! sticky_rename {
    ([$($file_types:ident $( ( $($args:expr),* ) )?),+], $fs: tt) => {
        $(sticky_rename!(@ $file_types $( ($($args),*) )?, $fs);)+
    };

    (@ $file_type:ident $( ( $($args:expr),* ) )?, [$($raw_ft: expr),+]) => {
        paste::paste! {
            $crate::test_case! {
                /// rename returns EACCES or EPERM if the file pointed at by the 'to' argument exists,
                /// the directory containing 'to' is marked sticky,
                /// and neither the containing directory nor 'to' are owned by the effective user ID
                [<rename_to_ $file_type:lower>], serialized, root => [$($raw_ft),+]
            }
            fn [<rename_to_ $file_type:lower>](ctx: &mut SerializedTestContext, to_ft: FileType) {
                assert_sticky_rename(ctx,
                    crate::runner::context::FileType::$file_type $( ( $($args),* ) )?,
                    Some(to_ft),
                    false)
            }

            // We also want to test when the `to` argument doesn't exist
            $crate::test_case! {
                /// rename returns EACCES or EPERM if the directory containing 'from' is marked sticky,
                /// and neither the containing directory nor 'from' are owned by the effective user ID
                [<rename_from_ $file_type:lower _none>], serialized, root
            }
            fn [<rename_from_ $file_type:lower _none>](ctx: &mut SerializedTestContext) {
                assert_sticky_rename(ctx, crate::runner::context::FileType::$file_type $( ( $($args),* ) )?, None, true)
            }

            $crate::test_case! {
                /// rename returns EACCES or EPERM if the directory containing 'from' is marked sticky,
                /// and neither the containing directory nor 'from' are owned by the effective user ID
                [<rename_from_ $file_type:lower>], serialized, root => [$($raw_ft),+]
            }
            fn [<rename_from_ $file_type:lower>](ctx: &mut SerializedTestContext, to_ft: FileType) {
                assert_sticky_rename(ctx,
                    crate::runner::context::FileType::$file_type $( ( $($args),* ) )?,
                    Some(to_ft),
                    true)
            }
        }
    };
}

/// Assert that rename returns EACCES or EPERM:
/// - if the file pointed at by the `to` argument exists (when `assert_from` is false),
/// - the directory containing 'from' (or `to` when `assert_from` is false) is marked sticky,
/// - and neither the containing directory nor 'from' (or `to` when `assert_from` is false)
/// are owned by the effective user ID.
fn assert_sticky_rename(
    ctx: &mut SerializedTestContext,
    from_ft: FileType,
    to_ft: Option<FileType>,
    assert_from: bool,
) {
    let user = ctx.get_new_user();
    let from_dir = ctx.create(FileType::Dir).unwrap();
    chown(&from_dir, Some(user.uid), Some(user.gid)).unwrap();

    let to_dir = ctx.create(FileType::Dir).unwrap();

    let sticky_dir = if assert_from { &from_dir } else { &to_dir };
    chmod(
        sticky_dir,
        Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX,
    )
    .unwrap();

    if assert_from {
        chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();
    }

    let from_path = from_dir.join("file");
    let to_path = to_dir.join("file");

    // User owns both: the sticky directory and the destination file.
    chown(sticky_dir, Some(user.uid), Some(user.gid)).unwrap();
    let from_path = ctx.create_named(from_ft.clone(), &from_path).unwrap();
    let metadata = lstat(&from_path).unwrap().as_time_invariant();
    // We create a file if to_ft is not None
    if let Some(to_ft) = to_ft.as_ref() {
        ctx.create_named(to_ft.clone(), &to_path).unwrap();
        lchown(&to_path, Some(user.uid), Some(user.gid)).unwrap();
    };

    ctx.as_user(&user, None, || {
        assert!(rename(&from_path, &to_path).is_ok());
    });
    assert!(!from_path.exists());
    let current_meta = lstat(&to_path).unwrap();
    assert_eq!(metadata, current_meta.as_time_invariant());

    ctx.as_user(&user, None, || {
        assert!(rename(&to_path, &from_path).is_ok());
    });
    assert!(!to_path.exists());
    let current_meta = lstat(&from_path).unwrap();
    assert_eq!(metadata, current_meta.as_time_invariant());

    //TODO: RAII
    unlink(&from_path).unwrap();

    let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
    let other_user = ctx.get_new_user();
    let different_users = &[current_user, other_user];

    // User owns the sticky directory, but doesn't own the destination file.
    chown(sticky_dir, Some(user.uid), Some(user.gid)).unwrap();
    for other_user in different_users {
        let from_path = ctx.create_named(from_ft.clone(), &from_path).unwrap();
        let from_owner = if assert_from { other_user } else { &user };
        lchown(&from_path, Some(from_owner.uid), Some(from_owner.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        let to_owner = if !assert_from { other_user } else { &user };
        if let Some(to_ft) = to_ft.as_ref() {
            ctx.create_named(to_ft.clone(), &to_path).unwrap();
            lchown(&to_path, Some(to_owner.uid), Some(to_owner.gid)).unwrap();
        };

        ctx.as_user(&user, None, || {
            assert!(rename(&from_path, &to_path).is_ok());
        });
        assert!(!from_path.exists());
        let current_meta = lstat(&to_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        ctx.as_user(&user, None, || {
            assert!(rename(&to_path, &from_path).is_ok());
        });
        assert!(!to_path.exists());
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());
        //TODO: RAII
        unlink(&from_path).unwrap();
    }

    // User owns the file, but doesn't own the sticky directory.
    for other_user in different_users {
        chown(sticky_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();

        let from_path = ctx.create_named(from_ft.clone(), &from_path).unwrap();
        lchown(&from_path, Some(user.uid), Some(user.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        if let Some(to_ft) = to_ft.as_ref() {
            ctx.create_named(to_ft.clone(), &to_path).unwrap();
            lchown(&to_path, Some(user.uid), Some(user.gid)).unwrap();
        };

        ctx.as_user(&user, None, || {
            assert!(rename(&from_path, &to_path).is_ok());
        });
        assert!(!from_path.exists());
        let current_meta = lstat(&to_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        ctx.as_user(&user, None, || {
            assert!(rename(&to_path, &from_path).is_ok());
        });
        assert!(!to_path.exists());
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());
        //TODO: RAII
        unlink(&from_path).unwrap();
    }

    // User doesn't own the sticky directory nor the file.
    for other_user in different_users {
        chown(sticky_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();

        let from_path = ctx.create_named(from_ft.clone(), &from_path).unwrap();
        let from_owner = if assert_from { other_user } else { &user };
        lchown(&from_path, Some(from_owner.uid), Some(from_owner.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        let to_owner = if !assert_from { other_user } else { &user };
        if let Some(to_ft) = to_ft.as_ref() {
            ctx.create_named(to_ft.clone(), &to_path).unwrap();
            lchown(&to_path, Some(to_owner.uid), Some(to_owner.gid)).unwrap();
        };

        ctx.as_user(&user, None, || {
            assert!(matches!(
                rename(&from_path, &to_path),
                Err(Errno::EACCES | Errno::EPERM)
            ));
        });
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        if to_ft.is_some() {
            let current_to_meta = lstat(&to_path).unwrap();
            assert_eq!(current_to_meta.st_uid, to_owner.uid.as_raw());
            assert_eq!(current_to_meta.st_gid, to_owner.gid.as_raw());
        }

        //TODO: RAII
        unlink(&from_path).unwrap();
        if to_ft.is_some() {
            unlink(&to_path).unwrap();
        }
    }
}

sticky_rename!(
    [Regular, Fifo, Block, Char, Socket, Symlink(None)],
    [Regular, Fifo, Block, Char, Socket, Symlink(None)]
);
