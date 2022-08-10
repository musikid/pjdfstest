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

pub fn rmdir<P: ?Sized + nix::NixPath>(path: &P) -> nix::Result<()> {
    let res = path.with_nix_path(|cstr| unsafe { nix::libc::rmdir(cstr.as_ptr()) })?;
    nix::errno::Errno::result(res).map(std::mem::drop)
}

pub const ALLPERMS: nix::sys::stat::mode_t = 0o777;

/// Wrapper for `renameat(None, path, None, path)`.
pub fn rename<P: ?Sized + nix::NixPath>(from: &P, to: &P) -> nix::Result<()> {
    nix::fcntl::renameat(None, from, None, to)
}

/// Wrapper for `linkat(None, path, None, path)`.
pub fn link<P: ?Sized + nix::NixPath>(from: &P, to: &P) -> nix::Result<()> {
    nix::unistd::linkat(
        None,
        from,
        None,
        to,
        nix::unistd::LinkatFlags::NoSymlinkFollow,
    )
}
