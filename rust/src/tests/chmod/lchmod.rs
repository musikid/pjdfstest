use super::*;
use crate::utils::lchmod;

crate::test_case! {
    /// lchmod returns EPERM if the operation would change the ownership, but the effective user ID is not the super-user
    // chmod/07.t
    eperm_not_owner, serialized, root
}
fn eperm_not_owner(ctx: &mut SerializedTestContext) {
    let current = User::from_uid(Uid::effective()).unwrap().unwrap();

    let user = ctx.get_new_user();
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();

    let file = ctx.create(FileType::Regular).unwrap();
    chown(&file, Some(user.uid), Some(user.gid)).unwrap();

    let mode = Mode::from_bits_truncate(0o642);
    let new_mode = Mode::from_bits_truncate(0o641);

    chown(&file, Some(user.uid), Some(user.gid)).unwrap();

    ctx.as_user(user, None, || {
        assert!(lchmod(&file, mode).is_ok());
        let file_stat = metadata(&file).unwrap();
        assert_eq!(file_stat.mode() as mode_t & ALLPERMS, mode.bits());
    });

    let other_user = ctx.get_new_user();
    ctx.as_user(other_user, None, || {
        assert_eq!(lchmod(&file, new_mode), Err(Errno::EPERM));
        let file_stat = metadata(&file).unwrap();
        assert_eq!(file_stat.mode() as mode_t & ALLPERMS, mode.bits());
    });
    chown(&file, Some(current.uid), Some(current.gid)).unwrap();

    ctx.as_user(user, None, || {
        assert_eq!(lchmod(&file, new_mode), Err(Errno::EPERM));
        let file_stat = metadata(&file).unwrap();
        assert_eq!(file_stat.mode() as mode_t & ALLPERMS, mode.bits());
    });
}

enotdir_comp_test_case!(lchmod(~path, Mode::empty()));
enoent_named_file_test_case!(lchmod(~path, Mode::empty()));
enoent_comp_test_case!(lchmod(~path, Mode::empty()));

// chmod/06.t#L25
eloop_comp_test_case!(lchmod(~path, Mode::empty()));

enametoolong_comp_test_case!(lchmod(~path, Mode::empty()));
enametoolong_path_test_case!(lchmod(~path, Mode::empty()));

// #[cfg(file_flags)]
mod flag {
    use crate::tests::errors::eperm::flag::immutable_append_named_test_case;

    use super::*;

    // chmod/08.t
    const EXPECTED_MODE: Mode = Mode::from_bits_truncate(0o100);
    immutable_append_named_test_case!(lchmod, |path| lchmod(path, EXPECTED_MODE), |path| metadata(
        path
    )
    .map_or(false, |m| m.mode() as mode_t & ALLPERMS
        == EXPECTED_MODE.bits()));
}

// chmod/09.t
erofs_named_test_case!(lchmod(~path, Mode::empty()));

// chmod/10.t
// TODO: lchmod is missing in libc
efault_path_test_case!(lchmod, |ptr| nix::libc::fchmodat(
    0,
    ptr,
    0,
    nix::libc::AT_SYMLINK_NOFOLLOW
));
