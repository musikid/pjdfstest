use std::path::Path;

use nix::{
    fcntl::renameat,
    sys::stat::{fchmodat, lstat, FchmodatFlags},
    unistd::{fchownat, linkat, symlinkat, FchownatFlags, Gid, LinkatFlags, Uid},
};

pub mod dev;

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)`.
pub fn chmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)
}

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::NoFollowSymlink)`.
pub fn lchmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    fchmodat(None, path, mode, FchmodatFlags::NoFollowSymlink)
}

/// Wrapper for `fchownat(None, path, mode, FchownatFlags::NoFollowSymlink)`.
pub fn lchown<P: ?Sized + nix::NixPath>(
    path: &P,
    owner: Option<Uid>,
    group: Option<Gid>,
) -> nix::Result<()> {
    fchownat(None, path, owner, group, FchownatFlags::NoFollowSymlink)
}

pub fn rmdir<P: ?Sized + nix::NixPath>(path: &P) -> nix::Result<()> {
    let res = path.with_nix_path(|cstr| unsafe { nix::libc::rmdir(cstr.as_ptr()) })?;
    nix::errno::Errno::result(res).map(std::mem::drop)
}

pub const ALLPERMS: nix::sys::stat::mode_t = 0o7777;

/// Wrapper for `renameat(None, old_path, None, new_path)`.
pub fn rename<P: ?Sized + nix::NixPath>(old_path: &P, new_path: &P) -> nix::Result<()> {
    renameat(None, old_path, None, new_path)
}

/// Wrapper for `linkat(None, old_path, None, new_path)`.
pub fn link<P: ?Sized + nix::NixPath>(old_path: &P, new_path: &P) -> nix::Result<()> {
    linkat(None, old_path, None, new_path, LinkatFlags::NoSymlinkFollow)
}

/// Wrapper for `symlinkat(path1, None, path2)`.
pub fn symlink<P: ?Sized + nix::NixPath>(path1: &P, path2: &P) -> nix::Result<()> {
    symlinkat(path1, None, path2)
}

/// Get mountpoint.
pub fn get_mountpoint(base_path: &Path) -> Result<&Path, anyhow::Error> {
    let base_dev = lstat(base_path)?.st_dev;

    let mut mountpoint = base_path;
    loop {
        let current = match mountpoint.parent() {
            Some(p) => p,
            // Root
            _ => return Ok(mountpoint),
        };
        let current_dev = lstat(current)?.st_dev;

        if current_dev != base_dev {
            break;
        }

        mountpoint = current;
    }

    Ok(mountpoint)
}

/// Safe wrapper for `lchflags`.
#[cfg(lchflags)]
pub fn lchflags<P: ?Sized + nix::NixPath>(
    path: &P,
    flags: nix::sys::stat::FileFlag,
) -> nix::Result<()> {
    use nix::errno::Errno;
    let res =
        path.with_nix_path(|cstr| unsafe { nix::libc::lchflags(cstr.as_ptr(), flags.bits()) })?;

    Errno::result(res).map(drop)
}
