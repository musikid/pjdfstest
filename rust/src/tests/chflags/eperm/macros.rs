use std::path::Path;

use nix::{errno::Errno, sys::stat::lstat};

use crate::{
    context::{FileType, SerializedTestContext},
    flags::FileFlags,
    utils::lchown,
};

type ChflagsVariant<P = Path> = fn(&P, nix::sys::stat::FileFlag) -> nix::Result<()>;

/// Function to generate `immutable_append_nounlink_not_root` test case with different chflags variant.
pub(in super::super) fn immutable_append_nounlink_not_root_factory(
    variant_name: &str,
    chflags_variant: ChflagsVariant,
    ctx: &mut SerializedTestContext,
    ft: FileType,
) {
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
        assert!(chflags_variant(&file, flag.into()).is_ok());
        let set_flags = lstat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            let res = chflags_variant(&file, FileFlags::UF_NODUMP.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "{variant_name} has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            let res = chflags_variant(&file, FileFlags::UF_NODUMP.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "{variant_name} has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

/// Create a test case which asserts that chflags returns EPERM
/// when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK is set
/// and the user is not the super-user.
/// There is a form for each chflags variant, namely lchflags and chflags:
///
/// - The regular chflags variant, which takes the syscall as argument.
///
/// ```
/// immutable_append_nounlink_not_root_test_case!(chflags);
/// ```
///
/// - The lchflags variant which adds the symlink file type.
///
/// ```
/// immutable_append_nounlink_not_root_test_case!(lchflags, symlink);
/// ```
///
macro_rules! immutable_append_nounlink_not_root_test_case {
    ($syscall: ident, symlink) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), " returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK is set and the user is not the super-user")]
            // chflags/08.t
            immutable_append_nounlink_not_root, serialized, root;
            $crate::tests::supports_file_flags!(
                SF_IMMUTABLE,
                SF_APPEND,
                SF_NOUNLINK
            )
             => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
        }
        fn immutable_append_nounlink_not_root(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
           $crate::tests::chflags::eperm::macros::immutable_append_nounlink_not_root_factory(stringify!($syscall), $syscall, ctx, ft)
        }
    };

    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), " returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK is set and the user is not the super-user")]
            // chflags/08.t
            immutable_append_nounlink_not_root, serialized, root;
            $crate::tests::supports_file_flags!(
                SF_IMMUTABLE,
                SF_APPEND,
                SF_NOUNLINK
            )
             => [Regular, Dir, Fifo, Block, Char, Socket]
        }
        fn immutable_append_nounlink_not_root(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
           $crate::tests::chflags::eperm::macros::immutable_append_nounlink_not_root_factory(stringify!($syscall), $syscall, ctx, ft)
        }
    };
}

pub(in super::super) use immutable_append_nounlink_not_root_test_case;

/// Function to generate `set_immutable_append_nounlink_not_root` test case with different chflags variant.
pub(in super::super) fn set_immutable_append_nounlink_not_root_factory(
    variant_name: &str,
    syscall_variant: ChflagsVariant,
    ctx: &mut SerializedTestContext,
    ft: FileType,
) {
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
            let res = syscall_variant(&file, flag.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "{variant_name} has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            let res = syscall_variant(&file, flag.into());
            assert_eq!(
                res,
                Err(Errno::EPERM),
                "{variant_name} has returned {res:?} for flag {flag} while EPERM was expected"
            );
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

/// Create a test case which asserts that chflags returns EPERM
/// if non-super-user tries to set one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK.
/// There is a form for each chflags variant, namely lchflags and chflags:
///
/// - The regular chflags variant, which takes the syscall as argument.
///
/// ```
/// set_immutable_append_nounlink_not_root_test_case!(chflags);
/// ```
///
/// - The lchflags variant which adds the symlink file type.
///
/// ```
/// set_immutable_append_nounlink_not_root_test_case!(lchflags, symlink);
/// ```
///
macro_rules! set_immutable_append_nounlink_not_root_test_case {
    ($syscall: ident, symlink) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), "returns EPERM if non-super-user tries to set one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK")]
            // chflags/10.t
            set_immutable_append_nounlink_not_root, serialized, root;
            $crate::tests::supports_file_flags!(
                SF_IMMUTABLE,
                SF_APPEND,
                SF_NOUNLINK
            )
             => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
        }
        fn set_immutable_append_nounlink_not_root(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
            $crate::tests::chflags::eperm::macros::set_immutable_append_nounlink_not_root_factory(stringify!($syscall), $syscall, ctx, ft);
        }
    };

    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), "returns EPERM if non-super-user tries to set one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK")]
            // chflags/10.t
            set_immutable_append_nounlink_not_root, serialized, root;
            $crate::tests::supports_file_flags!(
                SF_IMMUTABLE,
                SF_APPEND,
                SF_NOUNLINK
            )
             => [Regular, Dir, Fifo, Block, Char, Socket]
        }
        fn set_immutable_append_nounlink_not_root(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
            $crate::tests::chflags::eperm::macros::set_immutable_append_nounlink_not_root_factory(stringify!($syscall), $syscall, ctx, ft);
        }
    };
}

pub(in super::super) use set_immutable_append_nounlink_not_root_test_case;

/// Function to generate `not_owner_not_root` test case with different chflags variant.
pub(in super::super) fn not_owner_not_root_factory(
    variant_name: &str,
    syscall_variant: ChflagsVariant,
    ctx: &mut SerializedTestContext,
    ft: FileType,
) {
    let file = ctx.create(ft).unwrap();
    let other_owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();

    let default_flags = lstat(&file).unwrap().st_flags;

    ctx.as_user(&not_owner, None, || {
        let res = syscall_variant(&file, FileFlags::UF_NODUMP.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "{variant_name} has returned {res:?} when trying with non-owner user and file owned by original owner while EPERM was expected"
        );
    });

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);

    lchown(&file, Some(other_owner.uid), Some(other_owner.gid)).unwrap();

    ctx.as_user(&not_owner, None, || {
        let res = syscall_variant(&file, FileFlags::UF_NODUMP.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "{variant_name} has returned {res:?} when trying with non-owner user and file owned by another user while EPERM was expected"
        );
    });

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);
}

/// Create a test case which asserts that chflags returns EPERM
/// when the effective user ID does not match the owner of the file
/// and the effective user ID is not the super-user.
/// There is a form for each chflags variant, namely lchflags and chflags:
///
/// - The regular chflags variant, which takes the syscall as argument.
///
/// ```
/// not_owner_not_root_test_case!(chflags);
/// ```
///
/// - The lchflags variant which adds the symlink file type.
///
/// ```
/// not_owner_not_root_test_case!(lchflags, symlink);
/// ```
///
macro_rules! not_owner_not_root_test_case {
    ($syscall: ident, symlink) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), " returns EPERM when the effective user ID does not match the owner of the file and the effective user ID is not the super-user")]
            // chflags/07.t
            not_owner_not_root, serialized, root
             => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
        }
        fn not_owner_not_root(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
            $crate::tests::chflags::eperm::macros::not_owner_not_root_factory(stringify!($syscall), $syscall, ctx, ft)
        }
    };

    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), " returns EPERM when the effective user ID does not match the owner of the file and the effective user ID is not the super-user")]
            // chflags/07.t
            not_owner_not_root, serialized, root
             => [Regular, Dir, Fifo, Block, Char, Socket]
        }
        fn not_owner_not_root(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
            $crate::tests::chflags::eperm::macros::not_owner_not_root_factory(stringify!($syscall), $syscall, ctx, ft)
        }
    };
}

pub(in super::super) use not_owner_not_root_test_case;

/// Function to generate `set_sf_snapshot_user` test case with different chflags variant.
pub(in super::super) fn set_sf_snapshot_user_factory(
    variant_name: &str,
    syscall_variant: ChflagsVariant,
    ctx: &mut SerializedTestContext,
    ft: FileType,
) {
    let file = ctx.create(ft).unwrap();
    let other_owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();

    let default_flags = lstat(&file).unwrap().st_flags;

    ctx.as_user(&not_owner, None, || {
        let res = syscall_variant(&file, FileFlags::SF_SNAPSHOT.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "{variant_name} has returned {res:?} when trying to set with non-owner user and file owned by original owner while EPERM was expected"
        );
    });

    let res = syscall_variant(&file, FileFlags::SF_SNAPSHOT.into());
    assert_eq!(
        res,
        Err(Errno::EPERM),
        "{variant_name} has returned {res:?} when trying to set with original owner and file owned by original owner while EPERM was expected"
    );

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);

    lchown(&file, Some(other_owner.uid), Some(other_owner.gid)).unwrap();

    ctx.as_user(&not_owner, None, || {
        let res = syscall_variant(&file, FileFlags::SF_SNAPSHOT.into());
        assert_eq!(
            res,
            Err(Errno::EPERM),
            "{variant_name} has returned {res:?} when trying to set with non-owner user and file owned by another user than the original owner while EPERM was expected"
        );
    });

    let res = syscall_variant(&file, FileFlags::SF_SNAPSHOT.into());
    assert_eq!(
        res,
        Err(Errno::EPERM),
        "{variant_name} has returned {res:?} when trying to set with original owner and file owned by another user than the original owner while EPERM was expected"
    );

    let flags = lstat(&file).unwrap().st_flags;
    assert_eq!(default_flags, flags);
}

/// Create a test case which asserts that chflags returns EPERM
/// when the effective user ID does not match the owner of the file
/// and the effective user ID is not the super-user.
/// There is a form for each chflags variant, namely lchflags and chflags:
///
/// - The regular chflags variant, which takes the syscall as argument.
///
/// ```
/// set_sf_snapshot_user_test_case!(chflags);
/// ```
///
/// - The lchflags variant which adds the symlink file type.
///
/// ```
/// set_sf_snapshot_user_test_case!(lchflags, symlink);
/// ```
///
macro_rules! set_sf_snapshot_user_test_case {
    ($syscall: ident, symlink) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), " returns EPERM if a user tries to set or remove the SF_SNAPSHOT flag")]
            // chflags/11.t
            set_sf_snapshot_user, serialized, root, crate::features::FileSystemFeature::ChflagsSfSnapshot
             => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
        }
        fn set_sf_snapshot_user(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
            $crate::tests::chflags::eperm::macros::set_sf_snapshot_user_factory(stringify!($syscall), $syscall, ctx, ft)
        }
    };

    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall), " returns EPERM if a user tries to set or remove the SF_SNAPSHOT flag")]
            // chflags/11.t
            set_sf_snapshot_user, serialized, root, crate::features::FileSystemFeature::ChflagsSfSnapshot
             => [Regular, Dir, Fifo, Block, Char, Socket]
        }
        fn set_sf_snapshot_user(ctx: &mut crate::context::SerializedTestContext, ft: crate::context::FileType) {
            $crate::tests::chflags::eperm::macros::set_sf_snapshot_user_factory(stringify!($syscall), $syscall, ctx, ft)
        }
    }
}

pub(in super::super) use set_sf_snapshot_user_test_case;
