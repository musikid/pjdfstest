use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{mknod, stat, Mode, SFlag},
    },
    unistd::{close, mkdir, mkfifo, pathconf, setegid, seteuid, setgroups, Gid, Uid, User},
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
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
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
        matches!(self, FileType::Block | FileType::Char)
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

    pub fn default_user() -> User {
        User::from_name("nobody").unwrap().unwrap()
    }

    //TODO: Maybe better as a macro? unwrap?
    /// Execute the function as another user/group.
    pub fn as_user<F>(&self, user: Option<String>, groups: Option<&[Gid]>, f: F)
    where
        F: FnMut(),
    {
        let user = user.map_or_else(SerializedTestContext::default_user, |name| {
            User::from_name(&name).unwrap().unwrap()
        });

        let original_euid = Uid::effective();
        let original_egid = Gid::effective();

        let groups = [std::slice::from_ref(&user.gid), groups.unwrap_or_default()].concat();
        setgroups(&groups).unwrap();

        setegid(user.gid).unwrap();
        seteuid(user.uid).unwrap();

        let res = catch_unwind(AssertUnwindSafe(f));

        seteuid(original_euid).unwrap();
        setegid(original_egid).unwrap();

        if let Err(e) = res {
            resume_unwind(e)
        }
    }
}

impl TestContext {
    /// Create a new test context.
    pub fn new<P: AsRef<Path>>(settings: &SettingsConfig, base_dir: P) -> Self {
        let naptime = Duration::from_secs_f64(settings.naptime);
        let temp_dir = tempdir_in(base_dir).unwrap();
        TestContext { naptime, temp_dir }
    }

    pub fn base_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create a regular file and open it.
    pub fn create_file(
        &self,
        oflag: OFlag,
        mode: Option<nix::sys::stat::mode_t>,
    ) -> Result<(PathBuf, RawFd), TestError> {
        let mut file = self.new_file(FileType::Regular);
        if let Some(mode) = mode {
            file = file.mode(mode);
        }
        Ok(file.open(oflag)?)
    }

    /// Return a file builder.
    pub fn new_file(&self, ft: FileType) -> FileBuilder {
        FileBuilder::new(ft, &self.base_path())
    }

    /// Create a file with a random name.
    pub fn create(&self, f_type: FileType) -> Result<PathBuf, TestError> {
        Ok(self.new_file(f_type).create()?)
    }

    /// Create a file whose name length is _PC_NAME_MAX.
    pub fn create_name_max(&self, f_type: FileType) -> Result<PathBuf, TestError> {
        let max_name_len = pathconf(self.base_path(), nix::unistd::PathconfVar::NAME_MAX)?.unwrap();

        let file = self.new_file(f_type).name(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(max_name_len as usize)
                .map(char::from)
                .collect::<String>(),
        );

        Ok(file.create()?)
    }

    /// Create a file whose path length is _PC_PATH_MAX.
    pub fn create_path_max(&self, f_type: FileType) -> Result<PathBuf, TestError> {
        let max_name_len =
            pathconf(self.base_path(), nix::unistd::PathconfVar::NAME_MAX)?.unwrap() as usize;
        let component_len = max_name_len / 2;

        // - 1 for null char
        let max_path_len =
            pathconf(self.base_path(), nix::unistd::PathconfVar::PATH_MAX)?.unwrap() as usize - 1;

        let mut path = self.base_path().to_owned();
        let initial_path_len = path.to_string_lossy().len();
        let remaining_chars = max_path_len - initial_path_len;

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

        self.new_file(f_type).name(&path).create()?;

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

/// Allows to create a file using builder pattern.
#[derive(Debug)]
pub struct FileBuilder {
    file_type: FileType,
    path: PathBuf,
    random_name: bool,
    mode: Mode,
}

impl FileBuilder {
    /// Create a file builder.
    pub fn new<P: AsRef<Path>>(file_type: FileType, base_path: &P) -> Self {
        Self {
            path: base_path.as_ref().to_path_buf(),
            random_name: true,
            mode: match &file_type {
                FileType::Dir => Mode::from_bits_truncate(0o755),
                _ => Mode::from_bits_truncate(0o644),
            },
            file_type,
        }
    }

    /// [`Take`](std::mem::take) and return the path final form.
    fn final_path(&mut self) -> PathBuf {
        if self.random_name {
            self.path.push(
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(NUM_RAND_CHARS)
                    .map(char::from)
                    .collect::<String>(),
            )
        }

        std::mem::take(&mut self.path)
    }

    /// Create the file according to the provided information.
    pub fn create(mut self) -> nix::Result<PathBuf> {
        let mode = self.mode;
        let path = self.final_path();

        match self.file_type {
            FileType::Regular => open(&path, OFlag::O_CREAT, mode).and_then(close),
            FileType::Dir => mkdir(&path, mode),
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
            FileType::Symlink(target) => symlink(
                target.as_deref().unwrap_or_else(|| Path::new("test")),
                &path,
            )
            .map_err(|e| nix::Error::try_from(e).unwrap_or(nix::errno::Errno::UnknownErrno)),
        }?;

        Ok(path)
    }

    /// Create the file according to the provided information and open it.
    /// This function automatically adds [`O_CREAT`](nix::fcntl::OFlag::O_CREAT) to the [`open`](nix::fcntl::open) flags when creating a regular file.
    pub fn open(mut self, oflags: OFlag) -> nix::Result<(PathBuf, RawFd)> {
        match self.file_type {
            FileType::Regular => {
                let path = self.final_path();
                open(&path, OFlag::O_CREAT | oflags, self.mode).map(|fd| (path, fd))
            }
            _ => self
                .create()
                .and_then(|p| open(&p, oflags, Mode::empty()).map(|fd| (p, fd))),
        }
    }

    /// Change file mode.
    pub fn mode(mut self, mode: nix::sys::stat::mode_t) -> Self {
        self.mode = Mode::from_bits_truncate(mode);
        self
    }

    /// Join `name` to the base path.
    /// An absolute path can also be provided, in this case it completely replaces the path.
    pub fn name<P: AsRef<Path>>(mut self, name: P) -> Self {
        self.path.push(name.as_ref());
        self.random_name = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use nix::{errno::Errno, fcntl::OFlag, sys::stat::Mode, unistd::pathconf};
    use tempfile::TempDir;
    use walkdir::WalkDir;

    use crate::{
        config::SettingsConfig,
        utils::{chmod, ALLPERMS},
    };

    use super::{FileType, TestContext};

    #[test]
    fn create() {
        for ft in [
            FileType::Regular,
            FileType::Dir,
            FileType::Fifo,
            FileType::Socket,
            FileType::Symlink(None),
        ] {
            let settings = SettingsConfig { naptime: 0. };
            let tempdir = TempDir::new().unwrap();
            let ctx = TestContext::new(&settings, tempdir.path());

            assert!(ctx.temp_dir.path().starts_with(tempdir.path()));

            let parent_content = WalkDir::new(tempdir.path())
                .min_depth(1)
                .into_iter()
                .collect::<Vec<_>>();
            assert_eq!(parent_content.len(), 1);
            assert!(parent_content[0].as_ref().unwrap().file_type().is_dir());
            assert_eq!(
                parent_content[0].as_ref().unwrap().path(),
                ctx.temp_dir.path()
            );
            assert_eq!(
                WalkDir::new(ctx.temp_dir.path())
                    .min_depth(1)
                    .into_iter()
                    .count(),
                0
            );

            let file = ctx.create(ft.clone()).unwrap();
            let parent_content: Vec<_> = WalkDir::new(tempdir.path())
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .collect();
            assert_eq!(parent_content.len(), 1);

            let content: Vec<_> = WalkDir::new(ctx.temp_dir.path())
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .collect();
            assert_eq!(content.len(), 1);
            assert_eq!(content[0].path(), &file);

            let file_stat = nix::sys::stat::lstat(&file).unwrap();
            assert_eq!(
                file_stat.st_mode & nix::libc::S_IFMT,
                match ft {
                    FileType::Dir => nix::libc::S_IFDIR,
                    FileType::Fifo => nix::libc::S_IFIFO,
                    FileType::Regular => nix::libc::S_IFREG,
                    FileType::Socket => nix::libc::S_IFSOCK,
                    FileType::Symlink(..) => nix::libc::S_IFLNK,
                    _ => unimplemented!(),
                }
            )
        }
    }

    #[test]
    fn name_max() {
        let tmpdir = TempDir::new().unwrap();
        let settings = SettingsConfig { naptime: 0. };
        let ctx = TestContext::new(&settings, &tmpdir.path());
        let file = ctx.create_name_max(FileType::Regular).unwrap();
        let name_len = file.file_name().unwrap().to_string_lossy().len();

        let max_len = pathconf(ctx.base_path(), nix::unistd::PathconfVar::NAME_MAX)
            .unwrap()
            .unwrap();
        assert_eq!(name_len, max_len as usize);
        let mut invalid = file.clone();
        invalid.set_file_name(invalid.file_name().unwrap().to_string_lossy().into_owned() + "x");

        assert_eq!(
            chmod(&invalid, Mode::empty()).unwrap_err(),
            Errno::ENAMETOOLONG
        );
    }

    #[test]
    fn path_max() {
        let tmpdir = TempDir::new().unwrap();
        let settings = SettingsConfig { naptime: 0. };
        let ctx = TestContext::new(&settings, &tmpdir.path());
        let file = ctx.create_path_max(FileType::Regular).unwrap();
        let path_len = file.to_string_lossy().len();

        // including null char
        let max_len = pathconf(ctx.base_path(), nix::unistd::PathconfVar::PATH_MAX)
            .unwrap()
            .unwrap()
            - 1;
        assert_eq!(path_len, max_len as usize);
        let mut invalid = file.clone();
        invalid.set_file_name(invalid.file_name().unwrap().to_string_lossy().into_owned() + "x");

        assert_eq!(
            chmod(&invalid, Mode::empty()).unwrap_err(),
            Errno::ENAMETOOLONG
        );
    }

    #[test]
    fn new_file() {
        let current_umask = nix::sys::stat::umask(Mode::from_bits_truncate(ALLPERMS));
        nix::sys::stat::umask(current_umask);

        for ft in [
            FileType::Regular,
            FileType::Dir,
            // FileType::Fifo,
            // FileType::Socket,
            // FileType::Symlink(None),
        ] {
            let tmpdir = TempDir::new().unwrap();
            let settings = SettingsConfig { naptime: 0. };
            let ctx = TestContext::new(&settings, &tmpdir.path());
            let name = "testing";
            let expected_mode = 0o725;
            let (path, _file) = ctx
                .new_file(ft)
                .mode(expected_mode)
                .name(name)
                .open(OFlag::O_RDONLY)
                .unwrap();

            assert_eq!(path.file_name().unwrap().to_string_lossy(), name);

            let file_stat = nix::sys::stat::stat(&path).unwrap();
            let actual_mode = file_stat.st_mode & ALLPERMS;
            assert_eq!(actual_mode, expected_mode & (!current_umask.bits()));
        }
    }

    #[test]
    fn regular_unique_syscall() {
        let tmpdir = TempDir::new().unwrap();
        let settings = SettingsConfig { naptime: 0. };
        let ctx = TestContext::new(&settings, &tmpdir.path());

        assert!(ctx
            .new_file(FileType::Regular)
            .mode(0o444)
            .open(OFlag::O_RDWR)
            .is_ok());
    }
}
