use std::path::Path;

use nix::{
    errno::Errno,
    sys::stat::{lstat, stat, FileFlag},
    unistd::{chflags, chown},
};

use crate::{
    config::Config,
    flags::FileFlags,
    runner::context::{FileType, SerializedTestContext},
    utils::{lchflags, lchown},
};

/// Fails when one of FileFlags::SF_IMMUTABLE, FileFlags::SF_APPEND or FileFlags::SF_NOUNLINK is unsupported.
fn supports_immutable_append_nounlink(config: &Config, _: &Path) -> anyhow::Result<()> {
    let flags = [
        FileFlags::SF_IMMUTABLE,
        FileFlags::SF_APPEND,
        FileFlags::SF_NOUNLINK,
    ];

    if !flags.iter().all(|f| config.features.file_flags.contains(f)) {
        anyhow::bail!("Need support for SF_IMMUTABLE, SF_APPEND, and SF_NOUNLINK flags");
    }

    Ok(())
}

crate::test_case! {
    /// chflags returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    /// is set and the user is not the super-user
    // chflags/08.t
    immutable_append_nounlink_not_root, serialized, root; supports_immutable_append_nounlink
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

    for flag in flags {
        assert!(lchflags(&file, flag).is_ok());
        let set_flags = stat(&file).unwrap().st_flags;

        ctx.as_user(&owner, None, || {
            assert_eq!(lchflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
        });

        let actual_flags = stat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

crate::test_case! {
    /// chflags returns EPERM when one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    /// is set and the user is not the super-user
    // chflags/08.t
    immutable_append_nounlink_not_root_symlink, serialized, root; supports_immutable_append_nounlink
}
fn immutable_append_nounlink_not_root_symlink(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();
    lchown(&file, Some(owner.uid), Some(owner.gid)).unwrap();

    let flags = [
        FileFlag::SF_IMMUTABLE,
        FileFlag::SF_APPEND,
        FileFlag::SF_NOUNLINK,
    ];

    for flag in flags {
        assert!(lchflags(&file, flag).is_ok());
        let set_flags = lstat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            assert_eq!(lchflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            assert_eq!(lchflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

crate::test_case! {
    /// chflags returns EPERM if non-super-user tries to set one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    // chflags/10.t
    set_immutable_append_nounlink_not_root, serialized, root; supports_immutable_append_nounlink
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

    for flag in flags {
        assert!(lchflags(&file, flag).is_ok());
        let set_flags = stat(&file).unwrap().st_flags;

        ctx.as_user(&owner, None, || {
            assert_eq!(lchflags(&file, FileFlag::UF_NODUMP), Err(Errno::EPERM));
        });

        let actual_flags = stat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}

crate::test_case! {
    /// chflags returns EPERM if non-super-user tries to set one of SF_IMMUTABLE, SF_APPEND, or SF_NOUNLINK
    // chflags/10.t
    set_immutable_append_nounlink_not_root_symlink, serialized, root; supports_immutable_append_nounlink
}
fn set_immutable_append_nounlink_not_root_symlink(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let owner = ctx.get_new_user();
    let not_owner = ctx.get_new_user();
    lchown(&file, Some(owner.uid), Some(owner.gid)).unwrap();

    let flags = [
        FileFlag::SF_IMMUTABLE,
        FileFlag::SF_APPEND,
        FileFlag::SF_NOUNLINK,
    ];

    for flag in flags {
        assert!(lchflags(&file, flag).is_ok());
        let set_flags = lstat(&file).unwrap().st_flags;

        ctx.as_user(&not_owner, None, || {
            assert_eq!(lchflags(&file, flag), Err(Errno::EPERM));
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);

        ctx.as_user(&owner, None, || {
            assert_eq!(lchflags(&file, flag), Err(Errno::EPERM));
        });

        let actual_flags = lstat(&file).unwrap().st_flags;
        assert_eq!(set_flags, actual_flags);
    }
}
