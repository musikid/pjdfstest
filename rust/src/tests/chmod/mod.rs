mod errno;
mod lchmod;
mod permission;

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)`.
fn chmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    nix::sys::stat::fchmodat(
        None,
        path,
        mode,
        nix::sys::stat::FchmodatFlags::FollowSymlink,
    )
}

crate::pjdfs_group!(chmod; permission::test_case, errno::test_case);
