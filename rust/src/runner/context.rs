use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{mknod, Mode, SFlag},
    },
    unistd::{close, mkdir, mkfifo, pathconf, setegid, seteuid, Gid, Uid},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    os::unix::{fs::symlink, prelude::RawFd},
    panic::{catch_unwind, resume_unwind, UnwindSafe},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use strum_macros::EnumIter;
use tempfile::{tempdir, TempDir};
use thiserror::Error;

use crate::test::TestError;

/// File type, mainly used with [TestContext::create].
#[derive(Debug, Clone, Eq, PartialEq, EnumIter)]
pub enum FileType {
    Regular,
    Dir,
    Fifo,
    Block,
    Char,
    Socket,
    Symlink(Option<PathBuf>),
}

impl FileType {
    pub const fn privileged(&self) -> bool {
        match self {
            FileType::Regular => false,
            FileType::Dir => false,
            //TODO: Not sure for FIFO
            FileType::Fifo => false,
            FileType::Block => true,
            FileType::Char => true,
            FileType::Socket => false,
            FileType::Symlink(..) => false,
        }
    }
}

const NUM_RAND_CHARS: usize = 32;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("nix error")]
    Nix(#[from] nix::Error),
}

pub struct TestContext<const SERIALIZED: bool = false> {
    naptime: Duration,
    temp_dir: TempDir,
}

pub type SerializedTestContext = TestContext<true>;

impl SerializedTestContext {
    //TODO: Maybe better as a macro? unwrap?
    /// Execute the function as another user/group.
    pub fn as_user<F>(&self, uid: Option<Uid>, gid: Option<Gid>, mut f: F)
    where
        F: FnMut() + UnwindSafe,
    {
        if uid.is_none() && gid.is_none() {
            return f();
        }

        let original_euid = Uid::effective();
        let original_egid = Gid::effective();

        if let Some(gid) = gid {
            setegid(gid).unwrap();
        }

        if let Some(uid) = uid {
            seteuid(uid).unwrap();
        }

        let res = catch_unwind(f);

        if uid.is_some() {
            seteuid(original_euid).unwrap();
        }

        if gid.is_some() {
            setegid(original_egid).unwrap();
        }

        //TODO: Should we resume?
        if let Err(e) = res {
            resume_unwind(e)
        }
    }
}

impl<const SER: bool> TestContext<SER> {
    // TODO: make it private when all code runner is in the good module
    // TODO: replace the `naptime` argument with `SettingsConfig` once the lib
    // is merged into the bin.
    pub fn new(naptime: &f64) -> Self {
        let naptime = Duration::from_secs_f64(*naptime);
        let temp_dir = tempdir().unwrap();
        TestContext { naptime, temp_dir }
    }

    /// Create a regular file and open it.
    pub fn create_file(
        &mut self,
        oflag: OFlag,
        mode: Option<Mode>,
    ) -> Result<(PathBuf, RawFd), TestError> {
        let path = self.create(FileType::Regular)?;
        let file = open(&path, oflag, mode.unwrap_or_else(|| Mode::empty()))?;
        Ok((path, file))
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
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(close),
            FileType::Dir => mkdir(&path, Mode::from_bits_truncate(0o755)),
            FileType::Fifo => mkfifo(&path, mode),
            FileType::Block => mknod(&path, SFlag::S_IFBLK, mode, 0),
            FileType::Char => mknod(&path, SFlag::S_IFCHR, mode, 0),
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
            FileType::Symlink(target) => symlink(
                target.as_deref().unwrap_or_else(|| Path::new("test")),
                &path,
            )
            .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }

    pub fn create_max(&mut self, f_type: FileType) -> Result<PathBuf, TestError> {
        //TODO: const?
        let max_name_len =
            pathconf(self.temp_dir.path(), nix::unistd::PathconfVar::NAME_MAX)?.unwrap();

        let path = self.temp_dir.path().join(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(max_name_len as usize)
                .map(char::from)
                .collect::<String>(),
        );

        let mode = Mode::from_bits_truncate(0o644);

        match f_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(close),
            FileType::Dir => mkdir(&path, Mode::from_bits_truncate(0o755)),
            FileType::Fifo => mkfifo(&path, mode),
            FileType::Block => mknod(&path, SFlag::S_IFBLK, mode, 0),
            FileType::Char => mknod(&path, SFlag::S_IFCHR, mode, 0),
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
            FileType::Symlink(target) => symlink(
                target.as_deref().unwrap_or_else(|| Path::new("test")),
                &path,
            )
            .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }

    /// Create a file in a temp folder with the given name.
    pub fn create_named<P: AsRef<Path>>(
        &mut self,
        f_type: FileType,
        name: P,
    ) -> Result<PathBuf, TestError> {
        let path = self.temp_dir.path().join(name.as_ref());

        let mode = Mode::from_bits_truncate(0o644);

        match f_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(close),
            FileType::Dir => mkdir(&path, Mode::from_bits_truncate(0o755)),
            FileType::Fifo => mkfifo(&path, Mode::from_bits_truncate(0o755)),
            FileType::Block => mknod(&path, SFlag::S_IFBLK, mode, 0),
            FileType::Char => mknod(&path, SFlag::S_IFCHR, mode, 0),
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
            FileType::Symlink(target) => symlink(
                target.as_deref().unwrap_or_else(|| Path::new("test")),
                &path,
            )
            .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }

    /// A short sleep, long enough for file system timestamps to change.
    pub fn nap(&self) {
        thread::sleep(self.naptime)
    }
}
