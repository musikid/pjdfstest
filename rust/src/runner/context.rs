use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{makedev, mknod, Mode, SFlag},
    },
    unistd::{close, mkdir, mkfifo},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{os::unix::fs::symlink, path::PathBuf};
use strum_macros::EnumIter;
use tempfile::{tempdir, TempDir};
use thiserror::Error;

/// File type, mainly used with [TestContext::create].
#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum FileType {
    Regular,
    Dir,
    Fifo,
    Block(Option<(u64, u64)>),
    Char(Option<(u64, u64)>),
    Socket,
    Symlink,
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

    pub fn create(&mut self, f_type: FileType) -> Result<PathBuf, ContextError> {
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
            //TODO: error type
            FileType::Symlink => symlink("test", &path)
                .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }
        .map_err(ContextError::Nix)?;

        Ok(path)
    }

    pub fn create_named<S: Into<String>>(
        &mut self,
        f_type: FileType,
        name: S,
    ) -> Result<PathBuf, ContextError> {
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
            FileType::Symlink => symlink("test", &path)
                .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }
        .map_err(ContextError::Nix)?;

        Ok(path)
    }
}
