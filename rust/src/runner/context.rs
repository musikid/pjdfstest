use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{makedev, mknod, Mode, SFlag},
    },
    unistd::{close, mkdir, mkfifo, setegid, seteuid, Gid, Uid},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{os::unix::fs::symlink, path::PathBuf};
use strum_macros::EnumIter;
use tempfile::{tempdir, TempDir};
use thiserror::Error;

use crate::test::TestError;

/// File type, mainly used with [TestContext::create].
#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum FileType {
    Regular,
    Dir,
    Fifo,
    Block(Option<(u64, u64)>),
    Char(Option<(u64, u64)>),
    Socket,
    Symlink(Option<PathBuf>),
}

const NUM_RAND_CHARS: usize = 32;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("nix error")]
    Nix(#[from] nix::Error),
}

pub struct TestContext {
    temp_dir: TempDir,
}

impl TestContext {
    // TODO: make it private when all code runner is in the good module
    pub fn new() -> Self {
        let temp_dir = tempdir().unwrap();
        TestContext { temp_dir }
    }

    // pub(super) fn clean() -> Result<(), ContextError> {}

    //TODO: Maybe better as a macro?
    /// Execute the function as another user/group.
    pub fn as_user<F>(&self, uid: Option<Uid>, gid: Option<Gid>, mut f: F) -> Result<(), TestError>
    where
        F: FnMut() -> Result<(), TestError>,
    {
        if uid.is_none() && gid.is_none() {
            return f();
        }

        let original_euid = Uid::effective();
        let original_egid = Gid::effective();

        if let Some(gid) = gid {
            setegid(gid)?;
        }

        if let Some(uid) = uid {
            seteuid(uid)?;
        }

        let res = f();

        if uid.is_some() {
            seteuid(original_euid)?;
        }

        if gid.is_some() {
            setegid(original_egid)?;
        }

        res
    }

    /// Create a file in a temp folder with a random name.
    pub fn create(&mut self, f_type: FileType) -> Result<PathBuf, TestError> {
        let path = self.temp_dir.path().join(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(NUM_RAND_CHARS)
                .map(char::from)
                .collect::<String>(),
        );

        let mode = Mode::from_bits_truncate(0o644);

        match f_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(|fd| close(fd)),
            FileType::Dir => mkdir(&path, Mode::from_bits_truncate(0o755)),
            FileType::Fifo => mkfifo(&path, mode),
            FileType::Block(dev) => mknod(
                &path,
                SFlag::S_IFBLK,
                mode,
                dev.map_or(makedev(1, 2), |(major, minor)| makedev(major, minor)),
            ),
            FileType::Char(dev) => mknod(
                &path,
                SFlag::S_IFCHR,
                mode,
                dev.map_or(makedev(1, 2), |(major, minor)| makedev(major, minor)),
            ),
            FileType::Socket => {
                let fd = socket(
                    nix::sys::socket::AddressFamily::Unix,
                    nix::sys::socket::SockType::Stream,
                    SockFlag::empty(),
                    None,
                )?;
                let sockaddr = UnixAddr::new(&path)?;
                bind(fd, &sockaddr)
            }
            //TODO: error type?
            FileType::Symlink(target) => symlink(target.unwrap_or(PathBuf::from("test")), &path)
                .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }

    pub fn create_max(&mut self, f_type: FileType) -> Result<PathBuf, TestError> {
        //TODO: const?
        let max_name_len = pathconf(self.temp_dir.path(), nix::unistd::PathconfVar::NAME_MAX)?.unwrap();

        let path = self.temp_dir.path().join(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(max_name_len as usize)
                .map(char::from)
                .collect::<String>(),
        );

        let mode = Mode::from_bits_truncate(0o644);

        match f_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(|fd| close(fd)),
            FileType::Dir => mkdir(&path, Mode::from_bits_truncate(0o755)),
            FileType::Fifo => mkfifo(&path, mode),
            FileType::Block(dev) => mknod(
                &path,
                SFlag::S_IFBLK,
                mode,
                dev.map_or(0, |(major, minor)| makedev(major, minor)),
            ),
            FileType::Char(dev) => mknod(
                &path,
                SFlag::S_IFCHR,
                mode,
                dev.map_or(0, |(major, minor)| makedev(major, minor)),
            ),
            FileType::Socket => {
                let fd = socket(
                    nix::sys::socket::AddressFamily::Unix,
                    nix::sys::socket::SockType::Stream,
                    SockFlag::empty(),
                    None,
                )?;
                let sockaddr = UnixAddr::new(&path)?;
                bind(fd, &sockaddr)
            }
            //TODO: error type?
            FileType::Symlink(target) => symlink(target.unwrap_or(PathBuf::from("test")), &path)
                .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }


    /// Create a file in a temp folder with the given name.
    pub fn create_named<S: Into<String>>(
        &mut self,
        f_type: FileType,
        name: S,
    ) -> Result<PathBuf, TestError> {
        let path = self.temp_dir.path().join(name.into());

        let mode = Mode::from_bits_truncate(0o644);

        match f_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(|fd| close(fd)),
            FileType::Dir => mkdir(&path, Mode::from_bits_truncate(0o755)),
            FileType::Fifo => mkfifo(&path, Mode::from_bits_truncate(0o755)),
            FileType::Block(dev) => mknod(
                &path,
                SFlag::S_IFBLK,
                mode,
                dev.map_or(0, |(major, minor)| makedev(major, minor)),
            ),
            FileType::Char(dev) => mknod(
                &path,
                SFlag::S_IFCHR,
                mode,
                dev.map_or(0, |(major, minor)| makedev(major, minor)),
            ),
            FileType::Socket => {
                let fd = socket(
                    nix::sys::socket::AddressFamily::Unix,
                    nix::sys::socket::SockType::Stream,
                    SockFlag::empty(),
                    None,
                )?;
                let sockaddr = UnixAddr::new(&path)?;
                bind(fd, &sockaddr)
            }
            //TODO: error type
            FileType::Symlink(target) => symlink(target.unwrap_or(PathBuf::from("test")), &path)
                .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }
}
