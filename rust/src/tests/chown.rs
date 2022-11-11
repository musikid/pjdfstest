use std::{
    fs::{metadata, symlink_metadata},
    os::unix::prelude::MetadataExt,
};

use nix::{
    libc::{gid_t, uid_t},
    sys::stat::{mode_t, Mode},
    unistd::{chown, Uid, User},
};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    utils::{chmod, lchmod, lchown, ALLPERMS},
};

use super::{assert_ctime_changed, assert_ctime_unchanged, assert_symlink_ctime_unchanged};

use super::errors::enoent::{
    enoent_comp_test_case, enoent_named_file_test_case, enoent_symlink_named_file_test_case,
};
use super::errors::enotdir::enotdir_comp_test_case;

fn chown_wrapper(ctx: &mut TestContext, path: &std::path::Path) -> nix::Result<()> {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
}

enotdir_comp_test_case!(chown, chown_wrapper);

// chown/04.t
enoent_named_file_test_case!(chown, chown_wrapper);

// chown/04.t
enoent_comp_test_case!(chown, chown_wrapper);

// chown/04.t
enoent_symlink_named_file_test_case!(chown, chown_wrapper);

mod lchown {
    use std::path::Path;

    use super::*;

    fn lchown_wrapper<P: AsRef<Path>>(ctx: &mut TestContext, path: P) -> nix::Result<()> {
        let path = path.as_ref();
        let user = ctx.get_new_user();
        lchown(path, Some(user.uid), Some(user.gid))
    }

    enotdir_comp_test_case!(lchown, lchown_wrapper);

    enoent_named_file_test_case!(lchown, lchown_wrapper);

    enoent_comp_test_case!(lchown, lchown_wrapper);
}

const ALLPERMS_SETBITS: mode_t = ALLPERMS | Mode::S_ISUID.bits() | Mode::S_ISGID.bits();

crate::test_case! {
    /// super-user can always modify ownership
    root_modify_ownership, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn root_modify_ownership(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();

    let (user, group) = ctx.get_new_entry();
    assert!(chown(&file, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    let root = User::from_name("root").unwrap().unwrap();
    assert!(chown(&file, Some(root.uid), Some(root.gid)).is_ok());

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, root.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, root.gid.as_raw());

    // Test on symlink

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let link_stat = symlink_metadata(&link).unwrap();

    assert!(chown(&link, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    let no_follow_link_stat = symlink_metadata(&link).unwrap();
    assert_eq!(no_follow_link_stat.uid() as uid_t, link_stat.uid() as uid_t);
    assert_eq!(no_follow_link_stat.gid() as gid_t, link_stat.gid() as gid_t);

    // lchown

    let file = ctx.create(ft).unwrap();

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    let symlink_stat = symlink_metadata(&file).unwrap();
    assert_eq!(symlink_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(symlink_stat.gid() as gid_t, group.gid.as_raw());
}

crate::test_case! {
    /// super-user can always modify ownership
    root_modify_ownership_symlink, serialized, root
}
fn root_modify_ownership_symlink(ctx: &mut SerializedTestContext) {
    let symlink = ctx.create(FileType::Symlink(None)).unwrap();

    let user = ctx.get_new_user();
    assert!(lchown(&symlink, Some(user.uid), Some(user.gid)).is_ok());

    let symlink_stat = symlink_metadata(&symlink).unwrap();
    assert_eq!(symlink_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(symlink_stat.gid() as gid_t, user.gid.as_raw());
}

crate::test_case! {
    /// non-super-user can modify file group if he is owner of a file and
    /// gid he is setting is in his groups list.
    owner_modify_ownership, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn owner_modify_ownership(ctx: &mut SerializedTestContext, ft: FileType) {
    let (user, group) = ctx.get_new_entry();
    let other_group = ctx.get_new_group();
    let another_group = ctx.get_new_group();

    let groups = &[other_group.gid, another_group.gid];

    let file = ctx.create(ft.clone()).unwrap();

    assert!(chown(&file, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(groups), || {
        assert!(chown(&file, None, Some(other_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(groups), || {
        assert!(chown(&file, Some(user.uid), Some(another_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, another_group.gid.as_raw());

    // We test if it follows symlink

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let link_stat = symlink_metadata(&link).unwrap();

    assert!(chown(&link, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(file_stat.uid() as uid_t, follow_link_stat.uid() as uid_t);
    assert_eq!(file_stat.gid() as gid_t, follow_link_stat.gid() as gid_t);

    let no_follow_link_stat = symlink_metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, no_follow_link_stat.uid() as uid_t);
    assert_eq!(link_stat.gid() as gid_t, no_follow_link_stat.gid() as gid_t);

    ctx.as_user(&user, Some(groups), || {
        assert!(chown(&link, None, Some(other_group.gid)).is_ok());
    });

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
    assert_eq!(file_stat.uid() as uid_t, follow_link_stat.uid() as uid_t);
    assert_eq!(file_stat.gid() as gid_t, follow_link_stat.gid() as gid_t);

    let no_follow_link_stat = symlink_metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, no_follow_link_stat.uid() as uid_t);
    assert_eq!(link_stat.gid() as gid_t, no_follow_link_stat.gid() as gid_t);

    ctx.as_user(&user, Some(groups), || {
        assert!(chown(&link, Some(user.uid), Some(another_group.gid)).is_ok());
    });

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, another_group.gid.as_raw());
    assert_eq!(file_stat.uid() as uid_t, follow_link_stat.uid() as uid_t);
    assert_eq!(file_stat.gid() as gid_t, follow_link_stat.gid() as gid_t);

    let no_follow_link_stat = symlink_metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, no_follow_link_stat.uid() as uid_t);
    assert_eq!(link_stat.gid() as gid_t, no_follow_link_stat.gid() as gid_t);

    // lchown

    let file = ctx.create(ft).unwrap();

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(groups), || {
        assert!(lchown(&file, None, Some(other_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(groups), || {
        assert!(lchown(&file, Some(user.uid), Some(another_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, another_group.gid.as_raw());
}

crate::test_case! {
    /// non-super-user can modify file group if he is owner of a file and
    /// gid he is setting is in his groups list.
    owner_modify_ownership_symlink, serialized, root
}
fn owner_modify_ownership_symlink(ctx: &mut SerializedTestContext) {
    let (user, group) = ctx.get_new_entry();
    let other_group = ctx.get_new_group();
    let another_group = ctx.get_new_group();

    let groups = &[other_group.gid, another_group.gid];

    let file = ctx.create(FileType::Symlink(None)).unwrap();

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(groups), || {
        assert!(lchown(&file, None, Some(other_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(groups), || {
        assert!(lchown(&file, Some(user.uid), Some(another_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, another_group.gid.as_raw());
}

crate::test_case! {
    /// chown(2) return 0 if user is not owner of a file, but chown(2) is called
    /// with both uid and gid equal to -1.
    not_owner_no_modification_success, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn not_owner_no_modification_success(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();

    let (user, group) = ctx.get_new_entry();
    let other_user = ctx.get_new_user();

    assert!(chown(&file, Some(user.uid), Some(group.gid)).is_ok());
    ctx.as_user(&other_user, None, || {
        assert!(chown(&file, None, None).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    // Test if it follows symlinks

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    let link_stat = symlink_metadata(&link).unwrap();

    assert!(chown(&link, Some(user.uid), Some(group.gid)).is_ok());

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    let no_follow_link_stat = symlink_metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, no_follow_link_stat.uid() as uid_t);
    assert_eq!(link_stat.gid() as gid_t, no_follow_link_stat.gid() as gid_t);

    ctx.as_user(&other_user, None, || {
        assert!(chown(&file, None, None).is_ok());
    });
    //TODO: Should test with the link?
    // ctx.as_user(&other_user, None, || {
    //     assert!(chown(&link, None, None).is_ok());
    // });

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    let no_follow_link_stat = symlink_metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, no_follow_link_stat.uid() as uid_t);
    assert_eq!(link_stat.gid() as gid_t, no_follow_link_stat.gid() as gid_t);

    // lchown
    let file = ctx.create(ft).unwrap();

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    ctx.as_user(&other_user, None, || {
        assert!(lchown(&file, None, None).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
}

crate::test_case! {
    /// chown(2) return 0 if user is not owner of a file, but chown(2) is called
    /// with both uid and gid equal to -1.
    not_owner_no_modification_success_symlink, serialized, root
}
fn not_owner_no_modification_success_symlink(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let (user, group) = ctx.get_new_entry();
    let other_user = ctx.get_new_user();

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    ctx.as_user(&other_user, None, || {
        assert!(lchown(&file, None, None).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
}

crate::test_case! {
    /// when super-user calls chown(2), set-uid and set-gid bits may be removed.
    root_remove_suid_sgid, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn root_remove_suid_sgid(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();

    let (user, group) = ctx.get_new_entry();
    let mode = Mode::from_bits_truncate(0o555) | Mode::S_ISUID | Mode::S_ISGID;
    assert!(chown(&file, Some(user.uid), Some(group.gid)).is_ok());
    chmod(&file, mode).unwrap();

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    let (other_user, other_group) = ctx.get_new_entry();
    assert!(chown(&file, Some(other_user.uid), Some(other_group.gid)).is_ok());

    let mode_without_setbits = mode & !Mode::S_ISUID & !Mode::S_ISGID;

    let file_stat = metadata(&file).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    chmod(&file, mode).unwrap();
    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    let root = User::from_name("root").unwrap().unwrap();
    assert!(chown(&file, Some(root.uid), Some(root.gid)).is_ok());

    let file_stat = metadata(&file).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, root.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, root.gid.as_raw());

    // We check that it follow symlink
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();
    assert!(chown(&link, Some(user.uid), Some(group.gid)).is_ok());
    chmod(&link, mode).unwrap();

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    assert!(chown(&link, Some(other_user.uid), Some(other_group.gid)).is_ok());

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    let actual_file_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    let actual_link_mode = follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_file_mode == mode.bits() || actual_file_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
    assert!(actual_link_mode == mode.bits() || actual_link_mode == mode_without_setbits.bits());
    assert_eq!(follow_link_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, other_group.gid.as_raw());

    chmod(&link, mode).unwrap();

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, other_group.gid.as_raw());

    assert!(chown(&link, Some(root.uid), Some(root.gid)).is_ok());

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    let actual_file_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    let actual_link_mode = follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_file_mode == mode.bits() || actual_file_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, root.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, root.gid.as_raw());
    assert!(actual_link_mode == mode.bits() || actual_link_mode == mode_without_setbits.bits());
    assert_eq!(follow_link_stat.uid() as uid_t, root.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, root.gid.as_raw());

    // We test lchown here

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        lchmod(&file, mode).unwrap();
    } else {
        chmod(&file, mode).unwrap();
    }

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    assert!(lchown(&file, Some(other_user.uid), Some(other_group.gid)).is_ok());
    let file_stat = symlink_metadata(&file).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        lchmod(&file, mode).unwrap();
    } else {
        chmod(&file, mode).unwrap();
    }

    assert!(lchown(&file, Some(root.uid), Some(root.gid)).is_ok());
    let file_stat = symlink_metadata(&file).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, root.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, root.gid.as_raw());
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
crate::test_case! {
    /// when super-user calls lchown(2), set-uid and set-gid bits may be removed.
    root_remove_suid_sgid_symlink, serialized, root
}
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
fn root_remove_suid_sgid_symlink(ctx: &mut SerializedTestContext) {
    let link = ctx.create(FileType::Symlink(None)).unwrap();

    let mode = Mode::from_bits_truncate(0o555) | Mode::S_ISUID | Mode::S_ISGID;
    let mode_without_setbits = mode & !Mode::S_ISUID & !Mode::S_ISGID;

    let (user, group) = ctx.get_new_entry();
    let (other_user, other_group) = ctx.get_new_entry();
    let root = User::from_name("root").unwrap().unwrap();

    assert!(lchown(&link, Some(user.uid), Some(group.gid)).is_ok());

    lchmod(&link, mode).unwrap();

    let file_stat = symlink_metadata(&link).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    assert!(lchown(&link, Some(other_user.uid), Some(other_group.gid)).is_ok());
    let file_stat = symlink_metadata(&link).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, other_user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    lchmod(&link, mode).unwrap();

    assert!(lchown(&link, Some(root.uid), Some(root.gid)).is_ok());
    let file_stat = symlink_metadata(&link).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, root.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, root.gid.as_raw());
}

#[cfg(not(target_os = "linux"))]
crate::test_case! {
    /// when non-super-user calls chown(2) successfully, set-uid and set-gid bits may
    /// be removed, except when both uid and gid are equal to -1.
    user_remove_suid_sgid, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
#[cfg(target_os = "linux")]
crate::test_case! {
    /// when non-super-user calls chown(2) successfully, set-uid and set-gid bits may
    /// be removed, except when both uid and gid are equal to -1.
    user_remove_suid_sgid, serialized, root => [Regular, Fifo, Block, Char, Socket]
}
fn user_remove_suid_sgid(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();
    let (user, group) = ctx.get_new_entry();
    let other_group = ctx.get_new_group();

    let mode = Mode::from_bits_truncate(0o555) | Mode::S_ISUID | Mode::S_ISGID;
    let mode_without_setbits = mode & !Mode::S_ISUID & !Mode::S_ISGID;

    assert!(chown(&file, Some(user.uid), Some(group.gid)).is_ok());
    chmod(&file, mode).unwrap();

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(chown(&file, Some(user.uid), Some(other_group.gid)).is_ok());
    });

    let file_stat = metadata(&file).unwrap();
    // TODO: Linux doesn't clear the SGID/SUID bits for directories, despite the description noted
    // Linux makes a destinction for behavior when an executable file vs a
    // non-executable file. From chmod(2):
    //
    //   When the owner or group of an executable file are changed by an
    //   unprivileged user the S_ISUID and S_ISGID mode bits are cleared.
    //
    // I believe in this particular case, the behavior's bugged.

    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    chmod(&file, mode).unwrap();

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(chown(&file, None, Some(group.gid)).is_ok());
    });
    let file_stat = metadata(&file).unwrap();

    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    chmod(&file, mode).unwrap();

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(chown(&file, None, None).is_ok());
    });
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    // Test symlink

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();
    assert!(chown(&link, Some(user.uid), Some(group.gid)).is_ok());
    chmod(&link, mode).unwrap();

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(chown(&link, Some(user.uid), Some(other_group.gid)).is_ok());
    });

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, other_group.gid.as_raw());

    chmod(&link, mode).unwrap();
    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(chown(&link, None, Some(group.gid)).is_ok());
    });

    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    chmod(&link, mode).unwrap();
    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    assert_eq!(
        follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode.bits()
    );
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(chown(&link, None, None).is_ok());
    });
    let file_stat = metadata(&file).unwrap();
    let follow_link_stat = metadata(&link).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode_without_setbits.bits() || actual_mode == mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
    let actual_mode = follow_link_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode_without_setbits.bits() || actual_mode == mode.bits());
    assert_eq!(follow_link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(follow_link_stat.gid() as gid_t, group.gid.as_raw());

    // lchown

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());
    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        lchmod(&file, mode).unwrap();
    } else {
        chmod(&file, mode).unwrap();
    }

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(lchown(&file, Some(user.uid), Some(other_group.gid)).is_ok());
    });

    let file_stat = metadata(&file).unwrap();
    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        lchmod(&file, mode).unwrap();
    } else {
        chmod(&file, mode).unwrap();
    }

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(lchown(&file, None, Some(group.gid)).is_ok());
    });
    let file_stat = metadata(&file).unwrap();
    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    if cfg!(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        lchmod(&file, mode).unwrap();
    } else {
        chmod(&file, mode).unwrap();
    }

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(lchown(&file, None, None).is_ok());
    });
    let file_stat = metadata(&file).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
crate::test_case! {
    /// when non-super-user calls chown(2) successfully, set-uid and set-gid bits may
    /// be removed, except when both uid and gid are equal to -1.
    user_remove_suid_sgid_symlink, serialized, root
}
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
fn user_remove_suid_sgid_symlink(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let (user, group) = ctx.get_new_entry();
    let other_group = ctx.get_new_group();

    let mode = Mode::from_bits_truncate(0o555) | Mode::S_ISUID | Mode::S_ISGID;
    let mode_without_setbits = mode & !Mode::S_ISUID & !Mode::S_ISGID;

    assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());

    lchmod(&file, mode).unwrap();

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(lchown(&file, Some(user.uid), Some(other_group.gid)).is_ok());
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    lchmod(&file, mode).unwrap();

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.mode() as mode_t & ALLPERMS_SETBITS, mode.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(lchown(&file, None, Some(group.gid)).is_ok());
    });
    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(
        file_stat.mode() as mode_t & ALLPERMS_SETBITS,
        mode_without_setbits.bits()
    );
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    lchmod(&file, mode).unwrap();

    ctx.as_user(&user, Some(&[group.gid, other_group.gid]), || {
        assert!(lchown(&file, None, None).is_ok());
    });
    let file_stat = symlink_metadata(&file).unwrap();
    let actual_mode = file_stat.mode() as mode_t & ALLPERMS_SETBITS;
    assert!(actual_mode == mode.bits() || actual_mode == mode_without_setbits.bits());
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());
}

crate::test_case! {
    /// successful chown(2) call (except uid and gid equal to -1) updates ctime.
    update_ctime_success, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn update_ctime_success(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let (user, group) = ctx.get_new_entry();
    let other_group = ctx.get_new_group();

    assert_ctime_changed(ctx, &file, || {
        assert!(chown(&file, Some(user.uid), Some(group.gid)).is_ok());
    });

    assert_ctime_changed(ctx, &file, || {
        ctx.as_user(&user, Some(&[other_group.gid]), || {
            assert!(chown(&file, Some(user.uid), Some(other_group.gid)).is_ok());
        });
    });
    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());

    // Test if it follows symlinks

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    assert_ctime_changed(ctx, &link, || {
        assert!(chown(&link, Some(user.uid), Some(group.gid)).is_ok());
    });
    let link_stat = metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(link_stat.gid() as gid_t, group.gid.as_raw());

    assert_ctime_changed(ctx, &link, || {
        ctx.as_user(&user, Some(&[other_group.gid]), || {
            assert!(chown(&link, Some(user.uid), Some(other_group.gid)).is_ok());
        });
    });
    let link_stat = metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(link_stat.gid() as gid_t, other_group.gid.as_raw());

    // lchown
    assert_ctime_changed(ctx, &file, || {
        assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());
    });
    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    assert_ctime_changed(ctx, &file, || {
        ctx.as_user(&user, Some(&[other_group.gid]), || {
            assert!(lchown(&file, Some(user.uid), Some(other_group.gid)).is_ok());
        });
    });
    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
crate::test_case! {
    /// successful chown(2) call (except uid and gid equal to -1) updates ctime.
    update_ctime_success_symlink, serialized, root
}
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
fn update_ctime_success_symlink(ctx: &mut SerializedTestContext) {
    use super::assert_symlink_ctime_changed;

    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let (user, group) = ctx.get_new_entry();
    let other_group = ctx.get_new_group();

    assert_symlink_ctime_changed(ctx, &file, || {
        assert!(lchown(&file, Some(user.uid), Some(group.gid)).is_ok());
    });
    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, group.gid.as_raw());

    assert_symlink_ctime_changed(ctx, &file, || {
        ctx.as_user(&user, Some(&[other_group.gid]), || {
            assert!(lchown(&file, Some(user.uid), Some(other_group.gid)).is_ok());
        });
    });
    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, user.uid.as_raw());
    assert_eq!(file_stat.gid() as gid_t, other_group.gid.as_raw());
}

// TODO: On Linux: According to POSIX: If both owner and group are -1, the times need not be updated.
#[cfg(not(target_os = "linux"))]
crate::test_case! {
    /// successful chown(2) with -1 parameters should not change ctime
    unchanged_ctime_no_params, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
#[cfg(not(target_os = "linux"))]
fn unchanged_ctime_no_params(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let file_stat = metadata(&file).unwrap();
    let uid = file_stat.uid() as uid_t;
    let gid = file_stat.gid() as gid_t;

    assert_ctime_unchanged(ctx, &file, || {
        assert!(chown(&file, None, None).is_ok());
    });
    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);

    // Test if it follows symlinks

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    assert_ctime_unchanged(ctx, &link, || {
        assert!(chown(&link, None, None).is_ok());
    });
    let link_stat = metadata(&link).unwrap();
    assert_eq!(link_stat.uid() as uid_t, uid);
    assert_eq!(link_stat.gid() as gid_t, gid);

    // lchown

    assert_ctime_unchanged(ctx, &file, || {
        assert!(lchown(&file, None, None).is_ok());
    });
    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);
}

#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
crate::test_case! {
    /// successful chown(2) with -1 parameters should not change ctime
    unchanged_ctime_no_params_symlink, serialized, root
}
#[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"))]
fn unchanged_ctime_no_params_symlink(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let file_stat = symlink_metadata(&file).unwrap();
    let uid = file_stat.uid() as uid_t;
    let gid = file_stat.gid() as gid_t;

    assert_symlink_ctime_unchanged(ctx, &file, || {
        assert!(lchown(&file, None, None).is_ok());
    });
    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);
}

crate::test_case! {
    /// unsuccessful chown(2) does not update ctime.
    unchanged_ctime_failed, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn unchanged_ctime_failed(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let file_stat = metadata(&file).unwrap();
    let uid = file_stat.uid() as uid_t;
    let gid = file_stat.gid() as gid_t;

    let user = ctx.get_new_user();

    assert_ctime_unchanged(ctx, &file, || {
        ctx.as_user(&user, None, || {
            assert!(chown(&file, Some(user.uid), None).is_err());
            assert!(chown(&file, None, Some(user.gid)).is_err());
            assert!(chown(&file, Some(user.uid), Some(user.gid)).is_err());
        });
    });

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);

    // follow symlink

    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();
    assert_symlink_ctime_unchanged(ctx, &link, || {
        ctx.as_user(&user, None, || {
            assert!(chown(&link, Some(user.uid), None).is_err());
            assert!(chown(&link, None, Some(user.gid)).is_err());
            assert!(chown(&link, Some(user.uid), Some(user.gid)).is_err());
        });
    });

    let file_stat = metadata(&link).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);

    // lchown

    assert_ctime_unchanged(ctx, &file, || {
        ctx.as_user(&user, None, || {
            assert!(lchown(&file, Some(user.uid), None).is_err());
            assert!(lchown(&file, None, Some(user.gid)).is_err());
            assert!(lchown(&file, Some(user.uid), Some(user.gid)).is_err());
        });
    });

    let file_stat = metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);
}

crate::test_case! {
    /// unsuccessful chown(2) does not update ctime.
    unchanged_ctime_failed_symlink, serialized, root
}
fn unchanged_ctime_failed_symlink(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Symlink(None)).unwrap();
    let file_stat = symlink_metadata(&file).unwrap();
    let uid = file_stat.uid() as uid_t;
    let gid = file_stat.gid() as gid_t;

    let user = ctx.get_new_user();

    assert_symlink_ctime_unchanged(ctx, &file, || {
        ctx.as_user(&user, None, || {
            assert!(lchown(&file, Some(user.uid), None).is_err());
            assert!(lchown(&file, None, Some(user.gid)).is_err());
            assert!(lchown(&file, Some(user.uid), Some(user.gid)).is_err());
        });
    });

    let file_stat = symlink_metadata(&file).unwrap();
    assert_eq!(file_stat.uid() as uid_t, uid);
    assert_eq!(file_stat.gid() as gid_t, gid);
}
