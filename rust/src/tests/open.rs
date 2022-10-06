use std::fs::{metadata, symlink_metadata, FileType};
use std::os::unix::prelude::MetadataExt;
use std::path::Path;

use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::uio::pwrite;
use nix::unistd::close;

use crate::runner::context::{SerializedTestContext, TestContext};

use super::mksyscalls::{assert_perms_from_mode_and_umask, assert_uid_gid};
use super::{assert_times_changed, assert_times_unchanged, ATIME, CTIME, MTIME};

// open/00.t

fn open_wrapper(path: &Path, mode: Mode) -> nix::Result<()> {
    open(path, OFlag::O_CREAT | OFlag::O_WRONLY, mode).and_then(close)
}

crate::test_case! {
    /// POSIX: (If O_CREAT is specified and the file doesn't exist) [...] the access
    /// permission bits of the file mode shall be set to the value of the third
    /// argument taken as type mode_t modified as follows: a bitwise AND is performed
    /// on the file-mode bits and the corresponding bits in the complement of the
    /// process' file mode creation mask. Thus, all bits in the file mode whose
    /// corresponding bit in the file mode creation mask is set are cleared.
    permission_bits_from_mode, serialized
}
fn permission_bits_from_mode(ctx: &mut SerializedTestContext) {
    assert_perms_from_mode_and_umask(ctx, open_wrapper, FileType::is_file);
}

crate::test_case! {
    /// POSIX: (If O_CREAT is specified and the file doesn't exist) [...] the user ID
    /// of the file shall be set to the effective user ID of the process; the group ID
    /// of the file shall be set to the group ID of the file's parent directory or to
    /// the effective group ID of the process [...]
    uid_gid_eq_euid_egid, serialized, root
}
fn uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    assert_uid_gid(ctx, open_wrapper);
}

crate::test_case! {
    /// POSIX: Upon successful completion, open(O_CREAT) shall mark for update the st_atime,
    /// st_ctime, and st_mtime fields of the directory. Also, the st_ctime and
    /// st_mtime fields of the directory that contains the new entry shall be marked
    /// for update.
    changed_time_fields_success
}
fn changed_time_fields_success(ctx: &mut TestContext) {
    let path = ctx.gen_path();

    assert_times_changed()
        .path(ctx.base_path(), CTIME | MTIME)
        .paths(ctx.base_path(), &path, ATIME | CTIME | MTIME)
        .execute(ctx, false, || {
            open_wrapper(&path, Mode::from_bits_truncate(0o755)).unwrap();
        });
}

crate::test_case! {
    /// open do not update parent directory ctime and mtime fields if
    /// the file previously existed.
    exists_no_update
}
fn exists_no_update(ctx: &mut TestContext) {
    let file = ctx
        .create(crate::runner::context::FileType::Regular)
        .unwrap();

    assert_times_unchanged()
        .path(ctx.base_path(), CTIME | MTIME)
        .execute(ctx, false, || {
            assert!(open_wrapper(&file, Mode::from_bits_truncate(0o755)).is_ok());
        });
}

crate::test_case! {
    /// open with O_TRUNC should truncate an exisiting file.
    open_trunc
}
fn open_trunc(ctx: &mut TestContext) {
    let file = ctx
        .create(crate::runner::context::FileType::Regular)
        .unwrap();
    std::fs::write(&file, "data".as_bytes()).unwrap();
    assert_times_changed()
        .path(&file, CTIME | MTIME)
        .execute(ctx, false, || {
            assert!(open(&file, OFlag::O_WRONLY | OFlag::O_TRUNC, Mode::empty())
                .and_then(close)
                .is_ok());
        });
    let size = metadata(&file).unwrap().size();
    assert_eq!(size, 0);
}

crate::test_case! {
    /// interact with > 2 GB files
    // open/25.t
    interact_2gb
}
fn interact_2gb(ctx: &mut TestContext) {
    let (path, fd) = ctx.create_file(OFlag::O_WRONLY, Some(0o755)).unwrap();
    const DATA: &str = "data";
    const GB: usize = 1024usize.pow(3);
    let offset = 2 * GB as i64 + 1;
    pwrite(fd, DATA.as_bytes(), offset).unwrap();
    let expected_size = offset as u64 + DATA.len() as u64;
    let size = symlink_metadata(&path).unwrap().size();
    assert_eq!(size, expected_size);
    close(fd).unwrap();

    let fd = open(&path, OFlag::O_RDONLY, Mode::empty()).unwrap();
    let mut buf = [0; DATA.len()];
    nix::sys::uio::pread(fd, &mut buf, offset).unwrap();
    assert_eq!(buf, DATA.as_bytes());
}

// POSIX states that open should return ELOOP, but FreeBSD returns EMLINK instead
#[cfg(not(target_os = "freebsd"))]
crate::test_case! {
    /// open returns ELOOP when O_NOFOLLOW was specified and the target is a symbolic link
    open_nofollow
}
#[cfg(target_os = "freebsd")]
crate::test_case! {
    /// open returns EMLINK when O_NOFOLLOW was specified and the target is a symbolic link
    open_nofollow
}
fn open_nofollow(ctx: &mut TestContext) {
    use crate::runner::context::FileType;
    use nix::errno::Errno;

    let link = ctx.create(FileType::Symlink(None)).unwrap();

    assert!(matches!(
        open(
            &link,
            OFlag::O_RDONLY | OFlag::O_CREAT | OFlag::O_NOFOLLOW,
            Mode::empty()
        ),
        Err(Errno::EMLINK | Errno::ELOOP)
    ));
    assert!(matches!(
        open(&link, OFlag::O_RDONLY | OFlag::O_NOFOLLOW, Mode::empty()),
        Err(Errno::EMLINK | Errno::ELOOP)
    ));
    assert!(matches!(
        open(&link, OFlag::O_RDONLY | OFlag::O_NOFOLLOW, Mode::empty()),
        Err(Errno::EMLINK | Errno::ELOOP)
    ));
    assert!(matches!(
        open(&link, OFlag::O_RDWR | OFlag::O_NOFOLLOW, Mode::empty()),
        Err(Errno::EMLINK | Errno::ELOOP)
    ));
}
