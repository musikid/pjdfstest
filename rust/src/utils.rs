/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)`.
pub fn chmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    nix::sys::stat::fchmodat(
        None,
        path,
        mode,
        nix::sys::stat::FchmodatFlags::FollowSymlink,
    )
}

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::NoFollowSymlink)`.
pub fn lchmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    nix::sys::stat::fchmodat(
        None,
        path,
        mode,
        nix::sys::stat::FchmodatFlags::NoFollowSymlink,
    )
}

pub const ALLPERMS: nix::sys::stat::mode_t = 0o777;

pub fn link<P: ?Sized + nix::NixPath>(old: &P, new: &P) -> nix::Result<()> {
    nix::unistd::linkat(
        None,
        old,
        None,
        new,
        nix::unistd::LinkatFlags::NoSymlinkFollow,
    )
}
