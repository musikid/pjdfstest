use nix::{
    fcntl::{open, OFlag},
    sys::{
        socket::{bind, socket, SockFlag, UnixAddr},
        stat::{lstat, mknod, mode_t, umask, Mode, SFlag},
    },
    unistd::{
        close, getgroups, mkdir, mkfifo, pathconf, setegid, seteuid, setgroups, Gid, Group, Uid,
        User,
    },
};

use rand::distributions::{Alphanumeric, DistString};
use std::{
    cell::Cell,
    fs::create_dir_all,
    ops::{Deref, DerefMut},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use strum_macros::EnumIter;

use crate::{
    config::{Config, DummyAuthEntry, FeaturesConfig},
    utils::{chmod, lchmod, symlink},
};

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
/// Auth entries which are composed of a [`User`] and its associated [`Group`].
/// Allows to retrieve the auth entries.
#[derive(Debug)]
pub struct DummyAuthEntries<'a> {
    entries: &'a [DummyAuthEntry],
    index: Cell<usize>,
}

impl<'a> DummyAuthEntries<'a> {
    pub fn new(entries: &'a [DummyAuthEntry]) -> Self {
        Self {
            entries,
            index: Cell::new(0),
        }
    }

    /// Returns a new entry.
    pub fn get_new_entry(&self) -> (&User, &Group) {
        let entry = self.entries.get(self.index.get()).unwrap();
        self.index.set(self.index.get() + 1);

        (&entry.user, &entry.group)
    }
}

pub struct TestContext<'a> {
    naptime: Duration,
    temp_dir: &'a Path,
    features_config: &'a FeaturesConfig,
    auth_entries: DummyAuthEntries<'a>,
    #[cfg(target_os = "freebsd")]
    jail: Option<jail::RunningJail>,
}

pub struct SerializedTestContext<'a> {
    ctx: TestContext<'a>,
}

impl<'a> Deref for SerializedTestContext<'a> {
    type Target = TestContext<'a>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'a> DerefMut for SerializedTestContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl<'a> SerializedTestContext<'a> {
    pub fn new(config: &'a Config, entries: &'a [DummyAuthEntry], base_dir: &'a Path) -> Self {
        Self {
            ctx: TestContext::new(config, entries, base_dir),
        }
    }

    /// Execute the function as another user/group(s).
    /// If `groups` is set to `None`, only the default group associated to the user will be used
    /// and the effective [`Gid`] will be this one.
    /// Otherwise, the first provided [`Gid`] will be the effective one
    /// and the others will be added with `setgroups`.
    pub fn as_user<F>(&self, user: &User, groups: Option<&[Gid]>, f: F)
    where
        F: FnOnce(),
    {
        let original_euid = Uid::effective();
        let original_egid = Gid::effective();
        let original_groups = getgroups().unwrap();

        let groups: Vec<_> = groups
            .unwrap_or_else(|| std::slice::from_ref(&user.gid))
            .to_vec();
        setgroups(&groups).unwrap();

        setegid(groups[0]).unwrap();
        seteuid(user.uid).unwrap();

        let res = catch_unwind(AssertUnwindSafe(f));

        seteuid(original_euid).unwrap();
        setegid(original_egid).unwrap();
        setgroups(&original_groups).unwrap();

        if let Err(e) = res {
            resume_unwind(e)
        }
    }

    /// Execute the function with another umask.
    pub fn with_umask<F>(&self, mask: mode_t, f: F)
    where
        F: FnOnce(),
    {
        let previous_mask = umask(Mode::from_bits_truncate(mask));

        let res = catch_unwind(AssertUnwindSafe(f));

        umask(previous_mask);

        if let Err(e) = res {
            resume_unwind(e)
        }
    }
}

impl<'a> Drop for SerializedTestContext<'a> {
    fn drop(&mut self) {
        umask(Mode::empty());
    }
}

impl<'a> TestContext<'a> {
    /// Create a new test context.
    pub fn new(config: &'a Config, entries: &'a [DummyAuthEntry], temp_dir: &'a Path) -> Self {
        let naptime = Duration::from_secs_f64(config.settings.naptime);
        TestContext {
            naptime,
            temp_dir,
            features_config: &config.features,
            auth_entries: DummyAuthEntries::new(entries),
            #[cfg(target_os = "freebsd")]
            jail: None,
        }
    }

    /// Return the base path for this context.
    pub fn base_path(&self) -> &Path {
        self.temp_dir
    }

    pub fn features_config(&self) -> &FeaturesConfig {
        self.features_config
    }

    /// Generate a random path.
    pub fn gen_path(&self) -> PathBuf {
        self.base_path()
            .join(Alphanumeric.sample_string(&mut rand::thread_rng(), NUM_RAND_CHARS))
    }

    /// Create a regular file and open it.
    pub fn create_file(
        &self,
        oflag: OFlag,
        mode: Option<nix::sys::stat::mode_t>,
    ) -> Result<(PathBuf, OwnedFd), nix::Error> {
        let mut file = self.new_file(FileType::Regular);
        if let Some(mode) = mode {
            file = file.mode(mode);
        }
        file.open(oflag)
    }

    /// Return a file builder.
    pub fn new_file(&self, ft: FileType) -> FileBuilder {
        FileBuilder::new(ft, &self.base_path())
    }

    /// Create a file with a random name.
    pub fn create(&self, f_type: FileType) -> Result<PathBuf, nix::Error> {
        self.new_file(f_type).create()
    }

    /// Create a file whose name length is _PC_NAME_MAX.
    pub fn create_name_max(&self, f_type: FileType) -> Result<PathBuf, nix::Error> {
        let max_name_len =
            pathconf(self.base_path(), nix::unistd::PathconfVar::NAME_MAX)?.unwrap() as usize;

        let file = self
            .new_file(f_type)
            .name(Alphanumeric.sample_string(&mut rand::thread_rng(), max_name_len));

        file.create()
    }

    /// Create a file whose path length is _PC_PATH_MAX.
    pub fn create_path_max(&self, f_type: FileType) -> Result<PathBuf, nix::Error> {
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
            .map(|_| Alphanumeric.sample_string(&mut rand::thread_rng(), component_len - 1))
            .collect();

        let remaining_chars = remaining_chars % component_len - 1;
        if remaining_chars > 0 {
            path.extend(parts);

            create_dir_all(&path).unwrap();

            path.push(Alphanumeric.sample_string(&mut rand::thread_rng(), remaining_chars));
        } else {
            path.extend(&parts[..parts.len() - 1]);

            create_dir_all(&path).unwrap();

            path.push(&parts[parts.len() - 1]);
        }

        self.new_file(f_type).name(&path).create()?;

        Ok(path)
    }

    /// Returns a new entry.
    pub fn get_new_entry(&self) -> (&User, &Group) {
        self.auth_entries.get_new_entry()
    }

    /// Returns a new user.
    /// Alias of `get_new_entry`.
    pub fn get_new_user(&self) -> &User {
        self.get_new_entry().0
    }

    /// Returns a new group.
    /// Alias of `get_new_entry`.
    pub fn get_new_group(&self) -> &Group {
        self.get_new_entry().1
    }

    /// A short sleep, long enough for file system timestamps to change.
    pub fn nap(&self) {
        thread::sleep(self.naptime)
    }

    /// Set this Context's jail, so it will be destroyed during teardown.
    #[cfg(target_os = "freebsd")]
    pub fn set_jail(&mut self, jail: jail::RunningJail) {
        self.jail = Some(jail)
    }
}

// We implement Drop to circumvent the errors which arise from unlinking a directory for which
// search or write permission is denied, or a flag denying delete for a file.
impl<'a> Drop for TestContext<'a> {
    fn drop(&mut self) {
        let iter = walkdir::WalkDir::new(self.base_path()).into_iter();
        for entry in iter {
            let entry = match entry {
                Ok(e) => e,
                _ => continue,
            };

            if cfg!(lchflags) || entry.file_type().is_dir() {
                let file_stat = match lstat(entry.path()) {
                    Ok(s) => s,
                    _ => continue,
                };

                let mode = Mode::S_IRWXU;
                if (file_stat.st_mode & mode.bits()) != mode.bits() {
                    let _ = lchmod(entry.path(), mode);
                }

                // We remove all flags
                // TODO: Some platforms do not support lchflags, write chflagsat alternative for those (openbsd, macos, ios?)
                #[cfg(lchflags)]
                {
                    use crate::utils::lchflags;
                    use nix::{libc::fflags_t, sys::stat::FileFlag};

                    if file_stat.st_flags != FileFlag::empty().bits() as fflags_t {
                        let _ = lchflags(entry.path(), FileFlag::empty());
                    }
                }

                // Shut down any jails
                #[cfg(target_os = "freebsd")]
                if let Some(jail) = self.jail.take() {
                    let _ = jail.kill();
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
    mode: Option<Mode>,
}

impl FileBuilder {
    /// Create a file builder.
    pub fn new<P: AsRef<Path>>(file_type: FileType, base_path: &P) -> Self {
        Self {
            path: base_path.as_ref().to_path_buf(),
            random_name: true,
            mode: None,
            file_type,
        }
    }

    /// [`Take`](std::mem::take) and return the path final form.
    fn final_path(&mut self) -> PathBuf {
        if self.random_name {
            self.path
                .push(Alphanumeric.sample_string(&mut rand::thread_rng(), NUM_RAND_CHARS))
        }

        std::mem::take(&mut self.path)
    }

    /// Create the file according to the provided information.
    pub fn create(mut self) -> nix::Result<PathBuf> {
        let mode = self.mode.unwrap_or_else(|| match self.file_type {
            FileType::Dir => Mode::from_bits_truncate(0o755),
            _ => Mode::from_bits_truncate(0o644),
        });
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
                bind(fd.as_raw_fd(), &sockaddr)?;
                if let Some(mode) = self.mode {
                    chmod(&path, mode)?;
                }
                Ok(())
            }
            FileType::Symlink(target) => {
                symlink(
                    target.as_deref().unwrap_or_else(|| Path::new("test")),
                    &path,
                )?;

                #[cfg(lchmod)]
                if let Some(mode) = self.mode {
                    lchmod(&path, mode)?;
                }

                Ok(())
            }
        }?;

        Ok(path)
    }

    /// Create the file according to the provided information and open it.
    /// This function automatically adds [`O_CREAT`](nix::fcntl::OFlag::O_CREAT) to the [`open`](nix::fcntl::open) flags when creating a regular file.
    pub fn open(mut self, oflags: OFlag) -> nix::Result<(PathBuf, OwnedFd)> {
        match self.file_type {
            FileType::Regular => {
                let path = self.final_path();
                open(
                    &path,
                    OFlag::O_CREAT | oflags,
                    self.mode.unwrap_or_else(|| Mode::from_bits_truncate(0o644)),
                )
                .map(|fd| (path, unsafe { OwnedFd::from_raw_fd(fd) }))
            }
            _ => self.create().and_then(|p| {
                open(&p, oflags, Mode::empty()).map(|fd| (p, unsafe { OwnedFd::from_raw_fd(fd) }))
            }),
        }
    }

    /// Change file mode.
    pub fn mode(mut self, mode: nix::sys::stat::mode_t) -> Self {
        self.mode = Some(Mode::from_bits_truncate(mode));
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
        config::Config,
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
            let config = Config::default();
            let tempdir = TempDir::new().unwrap();
            let ctx = TestContext::new(&config, &[], tempdir.path());

            assert!(ctx.base_path().starts_with(tempdir.path()));

            let file = ctx.create(ft.clone()).unwrap();
            let content: Vec<_> = WalkDir::new(ctx.base_path())
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
        let config = Config::default();
        let ctx = TestContext::new(&config, &[], tmpdir.path());
        let file = ctx.create_name_max(FileType::Regular).unwrap();
        let name_len = file.file_name().unwrap().to_string_lossy().len();

        let max_len = pathconf(ctx.base_path(), nix::unistd::PathconfVar::NAME_MAX)
            .unwrap()
            .unwrap();
        assert_eq!(name_len, max_len as usize);
        let mut invalid = file;
        invalid.set_file_name(invalid.file_name().unwrap().to_string_lossy().into_owned() + "x");

        assert_eq!(
            chmod(&invalid, Mode::empty()).unwrap_err(),
            Errno::ENAMETOOLONG
        );
    }

    #[test]
    fn path_max() {
        let tmpdir = TempDir::new().unwrap();
        let config = Config::default();
        let ctx = TestContext::new(&config, &[], tmpdir.path());
        let file = ctx.create_path_max(FileType::Regular).unwrap();
        let path_len = file.to_string_lossy().len();

        // including null char
        let max_len = pathconf(ctx.base_path(), nix::unistd::PathconfVar::PATH_MAX)
            .unwrap()
            .unwrap()
            - 1;
        assert_eq!(path_len, max_len as usize);
        let mut invalid = file;
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
            // TODO: Test other file types
            // FileType::Fifo,
            // FileType::Socket,
            // FileType::Symlink(None),
        ] {
            let tmpdir = TempDir::new().unwrap();
            let config = Config::default();
            let ctx = TestContext::new(&config, &[], tmpdir.path());
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
        let config = Config::default();
        let ctx = TestContext::new(&config, &[], tmpdir.path());

        assert!(ctx
            .new_file(FileType::Regular)
            .mode(0o000)
            .open(OFlag::O_RDWR)
            .is_ok());
    }
}
