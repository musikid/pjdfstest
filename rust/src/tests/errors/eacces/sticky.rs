use nix::{
    errno::Errno,
    sys::stat::{lstat, Mode},
    unistd::{chown, unlink, Uid, User},
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    tests::AsTimeInvariant,
    utils::{chmod, lchown, rename, rmdir, ALLPERMS},
};

//TODO: Refactor

crate::test_case! {
    /// unlink returns EACCES or EPERM if the directory containing the file is marked sticky, and neither the containing directory
    /// nor the file to be removed are owned by the effective user ID
    // unlink/11.t
    unlink_file_sticky_dir_file_not_euid, serialized, root => [Regular, Fifo, Block, Char, Socket, Symlink(None)]
}
fn unlink_file_sticky_dir_file_not_euid(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();

    // User owns both: the sticky directory and the file.
    chmod(
        ctx.base_path(),
        Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX,
    )
    .unwrap();
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
    let file = ctx.create(ft.clone()).unwrap();
    ctx.as_user(&user, None, || {
        assert!(unlink(&file).is_ok());
    });
    assert_eq!(lstat(&file), Err(Errno::ENOENT));

    let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
    let other_user = ctx.get_new_user();
    let different_users = &[current_user, other_user];

    // User owns the sticky directory, but doesn't own the file.
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
    for other_user in different_users {
        let file = ctx.create(ft.clone()).unwrap();
        lchown(&file, Some(other_user.uid), Some(other_user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            assert!(unlink(&file).is_ok());
        });
        assert_eq!(lstat(&file), Err(Errno::ENOENT));
    }

    // User owns the file, but doesn't own the sticky directory.
    for other_user in different_users {
        chown(ctx.base_path(), Some(other_user.uid), Some(other_user.gid)).unwrap();
        let file = ctx.create(ft.clone()).unwrap();
        lchown(&file, Some(user.uid), Some(user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            assert!(unlink(&file).is_ok());
        });
        assert_eq!(lstat(&file), Err(Errno::ENOENT));
    }

    // User doesn't own the sticky directory nor the file.
    for other_user in different_users {
        chown(ctx.base_path(), Some(other_user.uid), Some(other_user.gid)).unwrap();
        let file = ctx.create(ft.clone()).unwrap();
        lchown(&file, Some(other_user.uid), Some(other_user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            assert!(matches!(unlink(&file), Err(Errno::EACCES | Errno::EPERM)));
        });
    }
}

crate::test_case! {
    /// rmdir returns EACCES or EPERM if the directory containing the file is marked sticky, and neither the containing directory
    /// nor the file to be removed are owned by the effective user ID
    // rmdir/11.t
    rmdir_file_sticky_dir_file_not_euid, serialized, root
}
fn rmdir_file_sticky_dir_file_not_euid(ctx: &mut SerializedTestContext) {
    let user = ctx.get_new_user();
    chmod(
        ctx.base_path(),
        Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX,
    )
    .unwrap();
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();

    // User owns both: the sticky directory and the directory to be removed.
    let file = ctx.create(FileType::Dir).unwrap();
    chown(&file, Some(user.uid), Some(user.gid)).unwrap();
    ctx.as_user(&user, None, || {
        assert!(rmdir(&file).is_ok());
    });
    assert_eq!(lstat(&file), Err(Errno::ENOENT));

    let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
    let other_user = ctx.get_new_user();
    let different_users = &[current_user, other_user];

    // User owns the sticky directory, but doesn't own the directory to be removed.
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
    for other_user in different_users {
        let dir = ctx.create(FileType::Dir).unwrap();
        lchown(&dir, Some(other_user.uid), Some(other_user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            assert!(rmdir(&dir).is_ok());
        });
        assert_eq!(lstat(&dir), Err(Errno::ENOENT));
    }

    // User owns the directory to be removed, but doesn't own the sticky directory.
    for other_user in different_users {
        chown(ctx.base_path(), Some(other_user.uid), Some(other_user.gid)).unwrap();
        let dir = ctx.create(FileType::Dir).unwrap();
        lchown(&dir, Some(user.uid), Some(user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            assert!(rmdir(&dir).is_ok());
        });
        assert_eq!(lstat(&dir), Err(Errno::ENOENT));
    }

    // User doesn't own the sticky directory nor the directory to be removed.
    for other_user in different_users {
        chown(ctx.base_path(), Some(other_user.uid), Some(other_user.gid)).unwrap();
        let dir = ctx.create(FileType::Dir).unwrap();
        lchown(&dir, Some(other_user.uid), Some(other_user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            assert!(matches!(rmdir(&dir), Err(Errno::EACCES | Errno::EPERM)));
        });
    }
}

// TODO: Refactor
crate::test_case! {
    /// rename returns EACCES or EPERM if the file pointed at by the 'to' argument exists, the directory containing 'to' is marked sticky,
    /// and neither the containing directory nor 'to' are owned by the effective user ID
    // rename/09.t
    // TODO: How to handle the file types? Create a macro for it?
    rename_sticky_dir_to, serialized, root => [Regular, Fifo, Block, Char, Socket, Symlink(None)]
}
fn rename_sticky_dir_to(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    let from_dir = ctx.create(FileType::Dir).unwrap();
    chown(&from_dir, Some(user.uid), Some(user.gid)).unwrap();
    let from_path = ctx.create_named(ft, from_dir.join("file")).unwrap();
    lchown(&from_path, Some(user.uid), Some(user.gid)).unwrap();

    let to_dir = ctx.create(FileType::Dir).unwrap();
    chmod(&to_dir, Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX).unwrap();

    let to_path = to_dir.join("file");

    let fts = [
        FileType::Regular,
        FileType::Fifo,
        FileType::Block,
        FileType::Char,
        FileType::Socket,
        FileType::Symlink(None),
    ];

    // User owns both: the sticky directory and the destination file.
    chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();
    let metadata = lstat(&from_path).unwrap().as_time_invariant();
    for to_ft in &fts {
        let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
        lchown(&to_path, Some(user.uid), Some(user.gid)).unwrap();

        ctx.as_user(&user, None, || {
            assert!(rename(&from_path, &to_path).is_ok());
        });
        assert_eq!(lstat(&from_path), Err(Errno::ENOENT));
        let current_meta = lstat(&to_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());

        ctx.as_user(&user, None, || {
            assert!(rename(&to_path, &from_path).is_ok());
        });
        assert_eq!(lstat(&to_path), Err(Errno::ENOENT));
        let current_meta = lstat(&from_path).unwrap();
        assert_eq!(metadata, current_meta.as_time_invariant());
    }

    let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
    let other_user = ctx.get_new_user();
    let different_users = &[current_user, other_user];

    // User owns the sticky directory, but doesn't own the destination file.
    chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();
    for other_user in different_users {
        for to_ft in &fts {
            let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
            lchown(&to_path, Some(other_user.uid), Some(other_user.gid)).unwrap();

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
        }
    }

    // User owns the file, but doesn't own the sticky directory.
    for other_user in different_users {
        chown(&to_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();
        for to_ft in &fts {
            let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
            lchown(&to_path, Some(user.uid), Some(user.gid)).unwrap();

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
        }
    }

    // User doesn't own the sticky directory nor the file.
    for other_user in different_users {
        chown(&to_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();
        for to_ft in &fts {
            let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
            lchown(&to_path, Some(other_user.uid), Some(other_user.gid)).unwrap();

            ctx.as_user(&user, None, || {
                assert!(matches!(
                    rename(&from_path, &to_path),
                    Err(Errno::EACCES | Errno::EPERM)
                ));
            });
            let current_meta = lstat(&from_path).unwrap();
            assert_eq!(metadata, current_meta.as_time_invariant());
            // TODO: to remove
            unlink(&to_path).unwrap();
        }
    }
}

// crate::test_case! {
//     /// rename returns EACCES or EPERM if the file pointed at by the 'to' argument exists, the directory containing 'to' is marked sticky,
//     /// and neither the containing directory nor 'to' are owned by the effective user ID
//     // rename/09.t
//     // TODO: How to handle the file types?
//     rename_sticky_dir_to_dir, serialized, root
// }
// fn rename_sticky_dir_to_dir(ctx: &mut SerializedTestContext) {
//     let user = ctx.get_new_user();
//     let from_dir = ctx.create(FileType::Dir).unwrap();
//     chown(&from_dir, Some(user.uid), Some(user.gid)).unwrap();
//     let from_path = ctx
//         .create_named(FileType::Dir, from_dir.join("file"))
//         .unwrap();
//     chown(&from_path, Some(user.uid), Some(user.gid)).unwrap();
//     let metadata = lstat(&from_path).unwrap().as_time_invariant();

//     let to_dir = ctx.create(FileType::Dir).unwrap();
//     chmod(&to_dir, Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX).unwrap();
//     chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();

//     // User owns both: the sticky directory and the destination directory.
//     let to_path = to_dir.join("file");
//     ctx.as_user(&user, None, || {
//         assert!(rename(&from_path, &to_path).is_ok());
//     });

//     let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
//     let other_user = ctx.get_new_user();
//     let different_users = &[current_user, other_user];

//     // User owns the sticky directory, but doesn't own the destination file.
//     chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();
//     for other_user in different_users {
//         chown(&to_path, Some(other_user.uid), Some(other_user.gid)).unwrap();
//         ctx.as_user(&user, None, || {
//             assert!(rename(&from_path, &to_path).is_ok());
//         });
//     }
//     todo!()
// }

// crate::test_case! {
//     /// rename returns EACCES or EPERM if the file pointed at by the 'from' argument exists, the directory containing 'from' is marked sticky,
//     /// and neither the containing directory nor 'from' are owned by the effective user ID
//     // rename/10.t
//     // TODO: How to handle the file types?
//     rename_sticky_dir_from, serialized, root => [Regular, Fifo, Block, Char, Socket, Symlink(None)]
// }
// fn rename_sticky_dir_from(ctx: &mut SerializedTestContext, ft: FileType) {
//     todo!();
//     let user = ctx.get_new_user();
//     let from_dir = ctx.create(FileType::Dir).unwrap();
//     chown(&from_dir, Some(user.uid), Some(user.gid)).unwrap();
//     let from_path = ctx.create_named(ft, from_dir.join("file")).unwrap();
//     lchown(&from_path, Some(user.uid), Some(user.gid)).unwrap();

//     let to_dir = ctx.create(FileType::Dir).unwrap();
//     chmod(&to_dir, Mode::from_bits_truncate(ALLPERMS) | Mode::S_ISVTX).unwrap();

//     let to_path = to_dir.join("file");

//     let fts = [
//         FileType::Regular,
//         FileType::Fifo,
//         FileType::Block,
//         FileType::Char,
//         FileType::Socket,
//         FileType::Symlink(None),
//     ];

//     // User owns both: the sticky directory and the destination file.
//     chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();
//     let metadata = lstat(&from_path).unwrap().as_time_invariant();
//     for to_ft in &fts {
//         let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
//         lchown(&to_path, Some(user.uid), Some(user.gid)).unwrap();

//         ctx.as_user(&user, None, || {
//             assert!(rename(&from_path, &to_path).is_ok());
//         });
//         assert_eq!(lstat(&from_path), Err(Errno::ENOENT));
//         let current_meta = lstat(&to_path).unwrap();
//         assert_eq!(metadata, current_meta.as_time_invariant());

//         ctx.as_user(&user, None, || {
//             assert!(rename(&to_path, &from_path).is_ok());
//         });
//         assert_eq!(lstat(&to_path), Err(Errno::ENOENT));
//         let current_meta = lstat(&from_path).unwrap();
//         assert_eq!(metadata, current_meta.as_time_invariant());
//     }

//     let current_user = User::from_uid(Uid::effective()).unwrap().unwrap();
//     let other_user = ctx.get_new_user();
//     let different_users = &[current_user, other_user];

//     // User owns the sticky directory, but doesn't own the destination file.
//     chown(&to_dir, Some(user.uid), Some(user.gid)).unwrap();
//     for other_user in different_users {
//         for to_ft in &fts {
//             let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
//             lchown(&to_path, Some(other_user.uid), Some(other_user.gid)).unwrap();

//             ctx.as_user(&user, None, || {
//                 assert!(rename(&from_path, &to_path).is_ok());
//             });
//             assert!(!from_path.exists());
//             let current_meta = lstat(&to_path).unwrap();
//             assert_eq!(metadata, current_meta.as_time_invariant());

//             ctx.as_user(&user, None, || {
//                 assert!(rename(&to_path, &from_path).is_ok());
//             });
//             assert!(!to_path.exists());
//             let current_meta = lstat(&from_path).unwrap();
//             assert_eq!(metadata, current_meta.as_time_invariant());
//         }
//     }

//     // User owns the file, but doesn't own the sticky directory.
//     for other_user in different_users {
//         chown(&to_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();
//         for to_ft in &fts {
//             let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
//             lchown(&to_path, Some(user.uid), Some(user.gid)).unwrap();

//             ctx.as_user(&user, None, || {
//                 assert!(rename(&from_path, &to_path).is_ok());
//             });
//             assert!(!from_path.exists());
//             let current_meta = lstat(&to_path).unwrap();
//             assert_eq!(metadata, current_meta.as_time_invariant());

//             ctx.as_user(&user, None, || {
//                 assert!(rename(&to_path, &from_path).is_ok());
//             });
//             assert!(!to_path.exists());
//             let current_meta = lstat(&from_path).unwrap();
//             assert_eq!(metadata, current_meta.as_time_invariant());
//         }
//     }

//     // User doesn't own the sticky directory nor the file.
//     for other_user in different_users {
//         chown(&to_dir, Some(other_user.uid), Some(other_user.gid)).unwrap();
//         for to_ft in &fts {
//             let to_path = ctx.create_named(to_ft.clone(), &to_path).unwrap();
//             lchown(&to_path, Some(other_user.uid), Some(other_user.gid)).unwrap();

//             ctx.as_user(&user, None, || {
//                 assert!(matches!(
//                     rename(&from_path, &to_path),
//                     Err(Errno::EACCES | Errno::EPERM)
//                 ));
//             });
//             let current_meta = lstat(&from_path).unwrap();
//             assert_eq!(metadata, current_meta.as_time_invariant());
//             unlink(&to_path).unwrap();
//         }
//     }
// }
