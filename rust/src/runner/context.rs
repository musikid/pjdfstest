use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{mknod, stat, Mode, SFlag},
    },
    unistd::{close, mkdir, mkfifo, pathconf, setegid, seteuid, Gid, Uid},
};

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios"
))]
use nix::{sys::stat::FileFlag, unistd::chflags};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    fs::create_dir_all,
    ops::{Deref, DerefMut},
    os::unix::{fs::symlink, prelude::RawFd},
    panic::{catch_unwind, resume_unwind, UnwindSafe},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use strum_macros::EnumIter;
use tempfile::{tempdir_in, TempDir};
use thiserror::Error;

use crate::{config::SettingsConfig, test::TestError, utils::lchmod};

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

pub struct TestContext {
    naptime: Duration,
    temp_dir: TempDir,
}

pub struct SerializedTestContext(TestContext);

impl Deref for SerializedTestContext {
    type Target = TestContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerializedTestContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SerializedTestContext {
    pub fn new<P: AsRef<Path>>(settings: &SettingsConfig, base_dir: P) -> Self {
        Self(TestContext::new(settings, base_dir))
    }

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

impl TestContext {
    pub fn new<P: AsRef<Path>>(settings: &SettingsConfig, base_dir: P) -> Self {
        let naptime = Duration::from_secs_f64(settings.naptime);
        let temp_dir = tempdir_in(base_dir).unwrap();
        TestContext { naptime, temp_dir }
    }

    pub fn base_path(&self) -> &Path {
        self.temp_dir.path()
    }

    //TODO: Generify create functions
    /// Create a file with a custom name.
    pub fn create_named<P: AsRef<Path>>(
        &mut self,
        f_type: FileType,
        name: P,
    ) -> Result<PathBuf, TestError> {
        let path = self.temp_dir.path().join(name.as_ref());

        create_type(f_type, &path)?;

        Ok(path)
    }

    /// Create a regular file and open it.
    pub fn create_file(
        &mut self,
        oflag: OFlag,
        mode: Option<Mode>,
    ) -> Result<(PathBuf, RawFd), TestError> {
        let path = self.create(FileType::Regular)?;
        let file = open(&path, oflag, mode.unwrap_or_else(Mode::empty))?;
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

        create_type(f_type, &path)?;

        Ok(path)
    }

    /// Create a file whose name length is _PC_NAME_MAX.
    pub fn create_name_max(&mut self, f_type: FileType) -> Result<PathBuf, TestError> {
        let max_name_len =
            pathconf(self.temp_dir.path(), nix::unistd::PathconfVar::NAME_MAX)?.unwrap();

        let path = self.temp_dir.path().join(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(max_name_len as usize)
                .map(char::from)
                .collect::<String>(),
        );

        create_type(f_type, &path)?;

        Ok(path)
    }

    /// Create a file whose path length is _PC_PATH_MAX.
    pub fn create_path_max(&mut self, f_type: FileType) -> Result<PathBuf, TestError> {
        let max_name_len = pathconf(self.temp_dir.path(), nix::unistd::PathconfVar::NAME_MAX)?
            .unwrap() as usize
            - 1;
        let component_len = max_name_len / 2;

        let max_path_len = pathconf(self.temp_dir.path(), nix::unistd::PathconfVar::PATH_MAX)?
            .unwrap() as usize
            - 1;

        let mut path = self.temp_dir.path().to_owned();
        let initial_path = path.to_string_lossy().len();
        let remaining_chars = max_path_len - initial_path;

        let parts: Vec<_> = (0..remaining_chars / component_len)
            .into_iter()
            .map(|_| {
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(component_len - 1)
                    .map(char::from)
                    .collect::<String>()
            })
            .collect();

        let remaining_chars = remaining_chars % component_len - 1;
        if remaining_chars > 0 {
            path.extend(parts);

            create_dir_all(&path).unwrap();

            path.push(
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(remaining_chars)
                    .map(char::from)
                    .collect::<String>(),
            );
        } else {
            path.extend(&parts[..parts.len() - 1]);

            create_dir_all(&path).unwrap();

            path.push(&parts[parts.len() - 1]);
        }

        create_type(f_type, &path)?;

        Ok(path)
    }

    /// A short sleep, long enough for file system timestamps to change.
    pub fn nap(&self) {
        thread::sleep(self.naptime)
    }
}

// We implement Drop to circumvent the errors which arise from unlinking a directory for which
// search or write permission is denied, or a flag denying delete for a file.
impl Drop for TestContext {
    fn drop(&mut self) {
        let iter = walkdir::WalkDir::new(self.base_path()).into_iter();
        for entry in iter {
            let entry = match entry {
                Ok(e) => e,
                _ => continue,
            };

            if cfg!(any(
                target_os = "openbsd",
                target_os = "netbsd",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "macos",
                target_os = "ios"
            )) || entry.file_type().is_dir()
            {
                let file_stat = match stat(entry.path()) {
                    Ok(s) => s,
                    _ => continue,
                };

                let mode = Mode::S_IRWXU;
                if (file_stat.st_mode & mode.bits()) != mode.bits() {
                    let _ = lchmod(entry.path(), mode);
                }

                // We remove all flags
                #[cfg(any(
                    target_os = "openbsd",
                    target_os = "netbsd",
                    target_os = "freebsd",
                    target_os = "dragonfly",
                    target_os = "macos",
                    target_os = "ios"
                ))]
                if file_stat.st_flags != 0 {
                    let _ = chflags(entry.path(), FileFlag::empty());
                }
            }
        }
    }
}

fn create_type<P: AsRef<Path>>(f_type: FileType, path: P) -> nix::Result<()> {
    let path = path.as_ref();
    let mode = match f_type {
        FileType::Dir => Mode::from_bits_truncate(0o755),
        _ => Mode::from_bits_truncate(0o644),
    };

    match f_type {
        FileType::Regular => open(path, OFlag::O_CREAT, mode).and_then(close),
        FileType::Dir => mkdir(path, mode),
        FileType::Fifo => mkfifo(path, mode),
        FileType::Block => mknod(path, SFlag::S_IFBLK, mode, 0),
        FileType::Char => mknod(path, SFlag::S_IFCHR, mode, 0),
        FileType::Socket => {
            let fd = socket(
                nix::sys::socket::AddressFamily::Unix,
                nix::sys::socket::SockType::Stream,
                SockFlag::empty(),
                None,
            )?;
            let sockaddr = UnixAddr::new(path)?;
            bind(fd, &sockaddr)
        }
        //TODO: error type?
        FileType::Symlink(target) => symlink(
            target.as_deref().unwrap_or_else(|| Path::new("test")),
            &path,
        )
        .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
    }
}
