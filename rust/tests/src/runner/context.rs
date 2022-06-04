use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{makedev, mknod, Mode, SFlag},
    },
    unistd::{close, mkdir, mkfifo},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};
use thiserror::Error;

/// File type, mainly used with [].
#[derive(Debug)]
pub enum FileType {
    Regular,
    Dir,
    Fifo,
    Block,
    Char,
    Socket,
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

    pub fn create<S: Into<String>>(
        &mut self,
        f_type: FileType,
        dev: Option<(u64, u64)>,
        name: Option<S>,
    ) -> Result<PathBuf, ContextError> {
        let path = self.temp_dir.path().join(name.map_or_else(
            || {
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(NUM_RAND_CHARS)
                    .map(char::from)
                    .collect::<String>()
            },
            |name| name.into(),
        ));

        let mode = Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH;

        match f_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(|fd| close(fd)),
            FileType::Dir => mkdir(&path, Mode::S_IRWXU),
            FileType::Fifo => mkfifo(&path, Mode::S_IRWXU),
            FileType::Block => mknod(
                &path,
                SFlag::S_IFBLK,
                mode,
                dev.map_or(0, |(major, minor)| makedev(major, minor)),
            ),
            FileType::Char => mknod(
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
        }
        .map_err(ContextError::Nix)?;

        Ok(path.to_owned())
    }
}
