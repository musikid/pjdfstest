use std::path::Path;

use nix::{
    errno::Errno,
    sys::stat::{lstat, Mode},
    unistd::{chown, unlink, Uid, User},
};

use crate::{
    context::{FileType, SerializedTestContext},
    tests::AsTimeInvariant,
    utils::{chmod, lchown, rename, rmdir, ALLPERMS},
};

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
            fn [<rename_to_ $file_type:lower>](ctx: &mut crate::SerializedTestContext, to_ft: crate::context::FileType) {
                assert_sticky_rename(
                    ctx,
                    crate::context::FileType::$file_type $( ( $($args),* ) )?,
                    Some(to_ft),
                    false
                )
            }

            // We also want to test when the `to` argument doesn't exist
            $crate::test_case! {
                /// rename returns EACCES or EPERM if the directory containing 'from' is marked sticky,
                /// and neither the containing directory nor 'from' are owned by the effective user ID
                [<rename_from_ $file_type:lower _none>], serialized, root
            }
            fn [<rename_from_ $file_type:lower _none>](ctx: &mut crate::SerializedTestContext) {
                assert_sticky_rename(ctx, crate::context::FileType::$file_type $( ( $($args),* ) )?, None, true)
            }

            $crate::test_case! {
                /// rename returns EACCES or EPERM if the directory containing 'from' is marked sticky,
                /// and neither the containing directory nor 'from' are owned by the effective user ID
                [<rename_from_ $file_type:lower>], serialized, root => [$($raw_ft),+]
            }
            fn [<rename_from_ $file_type:lower>](ctx: &mut crate::SerializedTestContext, to_ft: crate::context::FileType) {
                assert_sticky_rename(
                    ctx,
                    crate::context::FileType::$file_type $( ( $($args),* ) )?,
                    Some(to_ft),
                    true
                )
            }
        }
    };
}

/// Assert that rename returns EACCES or EPERM:
///
/// - if the file pointed at by the `to` argument exists (when `assert_from` is false),
/// - the directory containing 'from' (or `to` when `assert_from` is false) is marked sticky,
/// - and neither the containing directory nor 'from' (or `to` when `assert_from` is false)
/// are owned by the effective user ID.
/// The function assumes that the current user is root.
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

    // The current user is the effective user ID which is assumed to be root.
    let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
    let other_user = ctx.get_new_user();
    let different_users = &[&current_user, other_user];

    assert_owner_sticky_file_ok(
        sticky_dir, user, ctx, &from_ft, &from_path, &to_ft, &to_path,
    );

    assert_owner_sticky_not_file_ok(
        assert_from,
        sticky_dir,
        user,
        ctx,
        &from_ft,
        &from_path,
        &to_ft,
        &to_path,
        different_users,
    );

    assert_owner_not_sticky_file_ok(
        sticky_dir,
        user,
        ctx,
        &from_ft,
        &from_path,
        &to_ft,
        &to_path,
        different_users,
    );

    assert_not_owner_sticky_not_file_error(
        assert_from,
        sticky_dir,
        user,
        ctx,
        &from_ft,
        &from_path,
        &to_ft,
        &to_path,
        different_users,
    );

    // assert_owner_sticky_dir_ok(assert_from, sticky_dir, user, ctx, &from_path, &to_path);

    // assert_owner_sticky_not_source_dir_error(
    //     sticky_dir,
    //     user,
    //     ctx,
    //     &from_path,
    //     &to_path,
    //     different_users,
    // );
}

/// This function asserts that rename works as expected either when:
///
/// - user owns both the source sticky directory and the source file,
/// - user owns both the sticky directory and the destination file.
fn assert_owner_sticky_file_ok<P: ?Sized + nix::NixPath + AsRef<Path>>(
    sticky_dir: &P,
    user: &User,
    ctx: &SerializedTestContext,
    from_ft: &FileType,
    from_path: &P,
    to_ft: &Option<FileType>,
    to_path: &P,
) {
    chown(sticky_dir, Some(user.uid), Some(user.gid)).unwrap();
    let from_path = ctx
        .new_file(from_ft.clone())
        .name(from_path)
        .create()
        .unwrap();
    let metadata = lstat(&from_path).unwrap().as_time_invariant();
    // We create a file if to_ft is not None
    if let Some(to_ft) = to_ft {
        ctx.new_file(to_ft.clone()).name(to_path).create().unwrap();
        lchown(to_path, Some(user.uid), Some(user.gid)).unwrap();
    };

    ctx.as_user(&user, None, || {
        assert!(rename(&*from_path, to_path.as_ref()).is_ok());
    });
    assert!(!from_path.exists());
    let current_meta = lstat(to_path).unwrap();
    assert_eq!(metadata, current_meta.as_time_invariant());

    ctx.as_user(&user, None, || {
        assert!(rename(to_path.as_ref(), &from_path).is_ok());
    });
    assert!(!to_path.as_ref().exists());
    let current_meta = lstat(&from_path).unwrap();
    assert_eq!(metadata, current_meta.as_time_invariant());

    // TODO: RAII
    unlink(&from_path).unwrap();
}

/// Depending on the value of the assert_from parameter, this function asserts that
/// rename works as expected either when:
///
/// - user owns the source file, but doesn't own the source sticky directory.
/// - user owns the sticky directory, but doesn't own the destination file.
fn assert_owner_sticky_not_file_ok<P: ?Sized + nix::NixPath + AsRef<Path>>(
    assert_from: bool,
    sticky_dir: &P,
    user: &User,
    ctx: &SerializedTestContext,
    from_ft: &FileType,
    from_path: &P,
    to_ft: &Option<FileType>,
    to_path: &P,
    different_users: &[&User],
) {
    chown(sticky_dir, Some(user.uid), Some(user.gid)).unwrap();
    for other_user in different_users {
        let from_path = ctx
            .new_file(from_ft.clone())
            .name(&from_path)
            .create()
            .unwrap();
        let from_owner = if assert_from { other_user } else { &user };
        lchown(&from_path, Some(from_owner.uid), Some(from_owner.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        let to_owner = if !assert_from { other_user } else { &user };
        if let Some(to_ft) = to_ft.as_ref() {
            ctx.new_file(to_ft.clone()).name(&to_path).create().unwrap();
            lchown(to_path, Some(to_owner.uid), Some(to_owner.gid)).unwrap();
        };

        ctx.as_user(&user, None, || {
            assert!(rename(&*from_path, to_path.as_ref()).is_ok());
        });
        assert!(!from_path.exists());
        let current_meta = lstat(to_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        ctx.as_user(&user, None, || {
            assert!(rename(to_path.as_ref(), &from_path).is_ok());
        });
        assert!(!to_path.as_ref().exists());
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        // TODO: RAII
        unlink(&from_path).unwrap();
    }
}

/// This function asserts that rename works as expected either when:
///
/// - user owns the source file, but doesn't own the source sticky directory,
/// - user owns the destination file, but doesn't own the sticky directory.
fn assert_owner_not_sticky_file_ok<P: ?Sized + nix::NixPath + AsRef<Path>>(
    sticky_dir: &P,
    user: &User,
    ctx: &SerializedTestContext,
    from_ft: &FileType,
    from_path: &P,
    to_ft: &Option<FileType>,
    to_path: &P,
    different_users: &[&User],
) {
    for other_user in different_users {
        chown(sticky_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();

        let from_path = ctx
            .new_file(from_ft.clone())
            .name(&from_path)
            .create()
            .unwrap();
        lchown(&from_path, Some(user.uid), Some(user.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        if let Some(to_ft) = to_ft.as_ref() {
            ctx.new_file(to_ft.clone()).name(&to_path).create().unwrap();
            lchown(to_path, Some(user.uid), Some(user.gid)).unwrap();
        };

        ctx.as_user(&user, None, || {
            assert!(rename(&*from_path, to_path.as_ref()).is_ok());
        });
        assert!(!from_path.exists());
        let current_meta = lstat(to_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        ctx.as_user(&user, None, || {
            assert!(rename(to_path.as_ref(), &from_path).is_ok());
        });
        assert!(!to_path.as_ref().exists());
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        // TODO: RAII
        unlink(&from_path).unwrap();
    }
}

/// Depending on the value of the assert_from parameter, this function asserts that
/// rename returns EACCES or EPERM when:
///
/// - user doesn't own the source sticky directory nor the source file,
/// - user doesn't own the sticky directory nor the destination file.
fn assert_not_owner_sticky_not_file_error<P: ?Sized + nix::NixPath + AsRef<Path>>(
    assert_from: bool,
    sticky_dir: &P,
    user: &User,
    ctx: &SerializedTestContext,
    from_ft: &FileType,
    from_path: &P,
    to_ft: &Option<FileType>,
    to_path: &P,
    different_users: &[&User],
) {
    for other_user in different_users {
        chown(sticky_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();

        let from_path = ctx
            .new_file(from_ft.clone())
            .name(&from_path)
            .create()
            .unwrap();
        let from_owner = if assert_from { other_user } else { &user };
        lchown(&from_path, Some(from_owner.uid), Some(from_owner.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        let to_owner = if !assert_from { other_user } else { &user };
        if let Some(to_ft) = to_ft.as_ref() {
            ctx.new_file(to_ft.clone()).name(&to_path).create().unwrap();
            lchown(to_path, Some(to_owner.uid), Some(to_owner.gid)).unwrap();
        };

        ctx.as_user(&user, None, || {
            assert!(matches!(
                rename(&*from_path, to_path.as_ref()),
                Err(Errno::EACCES | Errno::EPERM)
            ));
        });
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        if to_ft.is_some() {
            let current_to_meta = lstat(to_path).unwrap();
            assert_eq!(current_to_meta.st_uid, to_owner.uid.as_raw());
            assert_eq!(current_to_meta.st_gid, to_owner.gid.as_raw());
        }

        // TODO: RAII
        unlink(&from_path).unwrap();
        if to_ft.is_some() {
            unlink(to_path).unwrap();
        }
    }
}

/// Depending on the value of the assert_from parameter, this function asserts that
/// rename works as expected either when:
///
/// - user owns both the source sticky directory and the source directory,
/// - user owns both the sticky directory and the destination directory.
fn assert_owner_sticky_dir_ok<P: ?Sized + nix::NixPath + AsRef<Path>>(
    assert_from: bool,
    sticky_dir: &P,
    user: &User,
    ctx: &SerializedTestContext,
    from_path: &P,
    to_path: &P,
) {
    chown(sticky_dir, Some(user.uid), Some(user.gid)).unwrap();
    let from_path = ctx
        .new_file(FileType::Dir)
        .name(from_path)
        .create()
        .unwrap();
    let metadata = lstat(&from_path).unwrap().as_time_invariant();

    if assert_from {
        ctx.as_user(user, None, || {
            assert!(rename(&*from_path, to_path.as_ref()).is_ok());
        });

        assert!(!from_path.exists());
        let current_meta = lstat(to_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        ctx.as_user(user, None, || {
            assert!(rename(to_path.as_ref(), &*from_path).is_ok());
        });
    }

    ctx.as_user(user, None, || {
        let to_path = ctx.new_file(FileType::Dir).name(to_path).create().unwrap();
        assert!(rename(&from_path, &to_path).is_ok());
    });

    assert!(!from_path.exists());
    let current_meta = lstat(to_path).unwrap();
    assert_eq!(metadata, current_meta.as_time_invariant());

    // TODO: RAII
    rmdir(to_path).unwrap();
}

/// This function asserts that rename throws an error when
/// user owns the source sticky directory, but doesn't own the source file.
/// This fails when changing parent directory, because this will modify
/// source directory inode (the .. link in it), but we can still rename it
/// without changing its parent directory.
fn assert_owner_sticky_not_source_dir_error<P: ?Sized + nix::NixPath + AsRef<Path>>(
    sticky_dir: &P,
    user: &User,
    ctx: &SerializedTestContext,
    from_path: &P,
    to_path: &P,
    different_users: &[&User],
) {
    chown(sticky_dir, Some(user.uid), Some(user.gid)).unwrap();
    for other_user in different_users {
        let from_path = ctx
            .new_file(FileType::Dir)
            .name(from_path)
            .create()
            .unwrap();
        let from_owner = other_user;
        lchown(&from_path, Some(from_owner.uid), Some(from_owner.gid)).unwrap();
        let metadata = lstat(&from_path).unwrap().as_time_invariant();

        ctx.as_user(user, None, || {
            assert!(matches!(
                rename(&*from_path, to_path.as_ref()),
                Err(Errno::EACCES | Errno::EPERM)
            ));
        });

        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());
    }
}

sticky_rename!(
    [Regular, Fifo, Block, Char, Socket, Symlink(None)],
    [Regular, Fifo, Block, Char, Socket, Symlink(None)]
);
