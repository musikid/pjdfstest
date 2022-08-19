use nix::{
    fcntl::renameat,
    sys::{
        stat::{fchmodat, FchmodatFlags},
        statfs::statfs,
    },
    unistd::{fchownat, linkat, symlinkat, FchownatFlags, Gid, LinkatFlags, Uid},
};
use std::path::Path;

use crate::config::Config;

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)`.
pub fn chmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    fchmodat(None, path, mode, FchmodatFlags::FollowSymlink)
}

/// Wrapper for `fchmodat(None, path, mode, FchmodatFlags::NoFollowSymlink)`.
pub fn lchmod<P: ?Sized + nix::NixPath>(path: &P, mode: nix::sys::stat::Mode) -> nix::Result<()> {
    fchmodat(None, path, mode, FchmodatFlags::NoFollowSymlink)
}

pub fn rmdir<P: ?Sized + nix::NixPath>(path: &P) -> nix::Result<()> {
    let res = path.with_nix_path(|cstr| unsafe { nix::libc::rmdir(cstr.as_ptr()) })?;
    nix::errno::Errno::result(res).map(std::mem::drop)
}

pub const ALLPERMS: nix::sys::stat::mode_t = 0o777;

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

/// Safe wrapper for `lchflags`.
#[cfg(any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly"))]
pub fn lchflags<P: ?Sized + nix::NixPath>(
    path: &P,
    flags: nix::sys::stat::FileFlag,
) -> nix::Result<()> {
    use nix::errno::Errno;
    let res =
        path.with_nix_path(|cstr| unsafe { nix::libc::lchflags(cstr.as_ptr(), flags.bits()) })?;

    Errno::result(res).map(drop)
}

/// Wrapper for `fchownat(None, path, owner, group, FchownatFlags::NoFollowSymlink)`.
pub fn lchown<P: ?Sized + nix::NixPath>(
    path: &P,
    owner: Option<Uid>,
    group: Option<Gid>,
) -> nix::Result<()> {
    fchownat(None, path, owner, group, FchownatFlags::NoFollowSymlink)
}

pub fn get_mountpoint(base_path: &Path) -> Result<&Path, anyhow::Error> {
    let base_dev = nix::sys::stat::lstat(base_path)?.st_dev;

    let mut mountpoint = base_path;
    loop {
        let current = match mountpoint.parent() {
            Some(p) => p,
            // Root
            _ => return Ok(mountpoint),
        };
        let current_dev = nix::sys::stat::lstat(current)?.st_dev;

        if current_dev != base_dev {
            break;
        }

        mountpoint = current;
    }

    Ok(mountpoint)
}

const REMAINING_SPACE_LIMIT: i64 = 128 * 1024i64.pow(2);

/// Guard to check that the file system is smaller than the fixde limit.
// TODO: Add a guard for mountpoint?
pub fn is_small(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    // TODO: Switch to portable one? seems to give errrneous values on FreeBSD
    let stat = statfs(base_path)?;
    let available_blocks: i64 = stat.blocks_available().try_into()?;
    let frag_size: i64 = match stat.block_size().try_into()? {
        0 => anyhow::bail!("Cannot get file system fragment size"),
        num => num,
    };
    let remaining_space: i64 = available_blocks * frag_size;

    if remaining_space >= REMAINING_SPACE_LIMIT {
        anyhow::bail!("File system free space is too high")
    }

    Ok(())
}
