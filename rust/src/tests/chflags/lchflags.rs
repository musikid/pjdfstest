use nix::{errno::Errno, sys::stat::lstat};

use crate::{
    context::{FileType, SerializedTestContext},
    features::FileSystemFeature,
    flags::FileFlags,
    tests::supports_file_flags,
    utils::{lchflags, lchown},
};

crate::test_case! {
    /// lchflags returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    /// is set and the user is not the super-user
    // chflags/08.t
    immutable_append_nounlink_not_root, serialized, root;
    supports_file_flags!(
        SF_IMMUTABLE,
        SF_APPEND,
        SF_NOUNLINK
    )
     => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn immutable_append_nounlink_not_root(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();
    lchown(&file, Some(owner.uid), Some(owner.gid)).unwrap();

    let flags = [
        FileFlags::SF_IMMUTABLE,
        FileFlags::SF_APPEND,
        FileFlags::SF_NOUNLINK,
    ];

    for flag in flags {
        assert!(lchflags(&file, flag.into()).is_ok());
        let set_flags = lstat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            let res = lchflags(&file, FileFlags::UF_NODUMP.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "lchflags has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            let res = lchflags(&file, FileFlags::UF_NODUMP.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "lchflags has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

crate::test_case! {
    /// chflags returns EPERM if non-super-user tries to set one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    // chflags/10.t
    set_immutable_append_nounlink_not_root, serialized, root;
    supports_file_flags!(
        SF_IMMUTABLE,
        SF_APPEND,
        SF_NOUNLINK
    )
     => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn set_immutable_append_nounlink_not_root(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();
    lchown(&file, Some(owner.uid), Some(owner.gid)).unwrap();

    let flags = [
        FileFlags::SF_IMMUTABLE,
        FileFlags::SF_APPEND,
        FileFlags::SF_NOUNLINK,
    ];

    for flag in flags {
        let set_flags = lstat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            let res = lchflags(&file, flag.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "lchflags has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            let res = lchflags(&file, flag.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "lchflags has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

crate::test_case! {
    /// chflags returns EPERM when the effective user ID does not match
    /// the owner of the file and the effective user ID is not the super-user
    // chflags/07.t
    not_owner_not_root, serialized, root
     => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn not_owner_not_root(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let other_owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();

    let default_flags = lstat(&file).unwrap().st_flags;

    ctx.as_user(&not_owner, None, || {
        let res = lchflags(&file, FileFlags::UF_NODUMP.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "lchflags has returned {res:?} when trying with non-owner user
            and file owned by original owner while EPERM was expected"
        );
    });

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);

    lchown(&file, Some(other_owner.uid), Some(other_owner.gid)).unwrap();

    ctx.as_user(&not_owner, None, || {
        let res = lchflags(&file, FileFlags::UF_NODUMP.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "lchflags has returned {res:?} when trying with non-owner user
            and file owned by another user while EPERM was expected"
        );
    });

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);
}

crate::test_case! {
    /// chflags returns EPERM if a user tries to set or remove the SF_SNAPSHOT flag
    // chflags/11.t
    set_sf_snapshot_user, serialized, root, FileSystemFeature::ChflagsSfSnapshot
     => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn set_sf_snapshot_user(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let other_owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();

    let default_flags = lstat(&file).unwrap().st_flags;

    ctx.as_user(&not_owner, None, || {
        let res = lchflags(&file, FileFlags::SF_SNAPSHOT.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "lchflags has returned {res:?} when trying to set with non-owner user
            and file owned by original owner while EPERM was expected"
        );
    });

    let res = lchflags(&file, FileFlags::SF_SNAPSHOT.into());
    assert_eq!(
        res,
        Err(Errno::EPERM),
        "lchflags has returned {res:?} when trying to set with original owner
            and file owned by original owner while EPERM was expected"
    );

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);

    lchown(&file, Some(other_owner.uid), Some(other_owner.gid)).unwrap();

    ctx.as_user(&not_owner, None, || {
        let res = lchflags(&file, FileFlags::SF_SNAPSHOT.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "lchflags has returned {res:?} when trying to set with non-owner user
            and file owned by another user than the original owner while EPERM was expected"
        );
    });

    let res = lchflags(&file, FileFlags::SF_SNAPSHOT.into());
    assert_eq!(
        res,
        Err(Errno::EPERM),
        "lchflags has returned {res:?} when trying to set with original owner
            and file owned by another user than the original owner while EPERM was expected"
    );

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);
}
