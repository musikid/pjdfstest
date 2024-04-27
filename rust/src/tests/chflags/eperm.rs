use nix::{
    errno::Errno,
    sys::stat::{stat, FileFlag},
    unistd::{chflags, chown},
};

use crate::{
    context::{FileType, SerializedTestContext},
    features::FileSystemFeature,
    tests::supports_file_flags,
};

crate::test_case! {
    /// chflags returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    /// is set and the user is not the super-user
    // chflags/08.t
    immutable_append_nounlink_not_root, serialized, root;
    supports_file_flags!(
        SF_IMMUTABLE,
        SF_APPEND,
        SF_NOUNLINK
    )
     => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn immutable_append_nounlink_not_root(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();
    chown(&file, Some(owner.uid), Some(owner.gid)).unwrap();

    let flags = [
        FileFlag::SF_IMMUTABLE,
        FileFlag::SF_APPEND,
        FileFlag::SF_NOUNLINK,
    ];

    for flag in flags {
        assert!(chflags(&file, flag).is_ok());
        let set_flags = stat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            assert_eq!(chflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
        });

        let actual_flags = stat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            assert_eq!(chflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
        });

        let actual_flags = stat(&file).unwrap().st_flags;
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
     => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn set_immutable_append_nounlink_not_root(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();
    chown(&file, Some(owner.uid), Some(owner.gid)).unwrap();

    let flags = [
        FileFlag::SF_IMMUTABLE,
        FileFlag::SF_APPEND,
        FileFlag::SF_NOUNLINK,
    ];

    for flag in flags {
        let set_flags = stat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            assert_eq!(chflags(&file, flag), Err(Errno::EPERM));
        });

        let actual_flags = stat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            assert_eq!(chflags(&file, flag), Err(Errno::EPERM));
        });

        let actual_flags = stat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

crate::test_case! {
    /// chflags returns EPERM when the effective user ID does not match
    /// the owner of the file and the effective user ID is not the super-user
    // chflags/07.t
    not_owner_not_root, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn not_owner_not_root(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let other_owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();

    chflags(&file, FileFlag::empty()).unwrap();
    let default_flags = stat(&file).unwrap().st_flags;

    ctx.as_user(&not_owner, None, || {
        assert_eq!(chflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
    });

    let flags = stat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);

    chown(&file, Some(other_owner.uid), Some(other_owner.gid)).unwrap();
    chflags(&file, FileFlag::empty()).unwrap();

    ctx.as_user(&not_owner, None, || {
        assert_eq!(chflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
    });

    let flags = stat(&file).unwrap().st_flags;
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

    chflags(&file, FileFlag::empty()).unwrap();
    let default_flags = stat(&file).unwrap().st_flags;

    ctx.as_user(&not_owner, None, || {
        assert_eq!(chflags(&file, FileFlag::SF_SNAPSHOT), Err(Errno::EPERM));
    });

    assert_eq!(chflags(&file, FileFlag::SF_SNAPSHOT), Err(Errno::EPERM));

    let flags = stat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);

    chown(&file, Some(other_owner.uid), Some(other_owner.gid)).unwrap();
    chflags(&file, FileFlag::empty()).unwrap();

    ctx.as_user(&not_owner, None, || {
        assert_eq!(chflags(&file, FileFlag::SF_SNAPSHOT), Err(Errno::EPERM));
    });

    assert_eq!(chflags(&file, FileFlag::SF_SNAPSHOT), Err(Errno::EPERM));

    let flags = stat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);
}
