use std::{fs::FileType as StdFileType, os::unix::fs::FileTypeExt, path::Path};

use nix::errno::Errno;
use nix::sys::stat::{mknod, Mode, SFlag};

use crate::runner::context::{FileType, SerializedTestContext, TestContext};

use super::errors::eloop::eloop_comp_test_case;
use super::errors::enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case};
use super::errors::enoent::enoent_comp_test_case;
use super::errors::enotdir::enotdir_comp_test_case;
use super::mksyscalls::{assert_perms_from_mode_and_umask, assert_uid_gid};
use super::{assert_times_changed, ATIME, CTIME, MTIME};

fn mknod_wrapper(path: &Path, mode: Mode) -> nix::Result<()> {
    mknod(path, SFlag::S_IFIFO, mode, 0)
}

crate::test_case! {
    /// POSIX: The file permission bits of the new FIFO shall be initialized from
    /// mode. The file permission bits of the mode argument shall be modified by the
    /// process' file creation mask.
    permission_bits_from_mode, serialized
}
fn permission_bits_from_mode(ctx: &mut SerializedTestContext) {
    assert_perms_from_mode_and_umask(ctx, mknod_wrapper, StdFileType::is_fifo);
}

crate::test_case! {
    /// POSIX: The FIFO's user ID shall be set to the process' effective user ID.
    /// The FIFO's group ID shall be set to the group ID of the parent directory or to
    /// the effective group ID of the process.
    uid_gid_eq_euid_egid, serialized, root
}
fn uid_gid_eq_euid_egid(ctx: &mut SerializedTestContext) {
    assert_uid_gid(ctx, mknod_wrapper);
}

crate::test_case! {
    /// POSIX: Upon successful completion, mkfifo() shall mark for update the st_atime,
    /// st_ctime, and st_mtime fields of the file. Also, the st_ctime and
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
            mknod_wrapper(&path, Mode::from_bits_truncate(0o644)).unwrap();
        });
}

// mknod/01.t
enotdir_comp_test_case!(mknod(~path, SFlag::S_IFIFO, Mode::empty(), 0));

crate::test_case! {
    /// mknod returns ENOTDIR if a component of the path prefix is not a directory
    /// when trying to create char/block files
    enotdir_comp_char_block, root => [Regular, Fifo, Block, Char, Socket]
}
fn enotdir_comp_char_block(ctx: &mut TestContext, ft: FileType) {
    let base_path = ctx.create(ft).unwrap();
    let path = base_path.join("previous_not_dir");

    // mknod/01.t
    assert_eq!(
        mknod(&path, SFlag::S_IFCHR, Mode::empty(), 0).unwrap_err(),
        Errno::ENOTDIR
    );
    assert_eq!(
        mknod(&path, SFlag::S_IFBLK, Mode::empty(), 0).unwrap_err(),
        Errno::ENOTDIR
    );
}

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "illumos"))]
crate::test_case! {
    /// mknod create device files
    // mknod/11.t
    device_files, root => [Block, Char]
}
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "illumos"))]
fn device_files(ctx: &mut TestContext, ft: FileType) {
    use std::{
        fs::symlink_metadata,
        os::unix::prelude::{MetadataExt, PermissionsExt},
    };

    use crate::utils::{
        dev::{major, makedev, minor},
        ALLPERMS,
    };
    use nix::sys::stat::mode_t;

    let (argument, check): (SFlag, fn(&StdFileType) -> bool) = match ft {
        FileType::Block => (SFlag::S_IFBLK, StdFileType::is_block_device),
        FileType::Char => (SFlag::S_IFCHR, StdFileType::is_char_device),
        _ => unreachable!(),
    };

    let file = ctx.gen_path();

    let mode = 0o755;
    let major_num = 1;
    let minor_num = 2;

    assert!(mknod(
        &file,
        argument,
        Mode::from_bits_truncate(mode),
        makedev(major_num, minor_num)
    )
    .is_ok());

    let stat = symlink_metadata(&file).unwrap();
    assert_eq!(stat.permissions().mode() as mode_t & ALLPERMS, mode);
    assert_eq!(major(stat.rdev()) as u64, major_num as u64);
    assert_eq!(minor(stat.rdev()) as u64, minor_num as u64);
    assert!(check(&stat.file_type()));
}

crate::test_case! {
    /// mknod changes st_ctime and st_mtime of the parent directory
    /// and marks for update the st_atime, st_ctime and st_mtime fields
    /// of the new file
    changed_times_success, root => [Block, Char]
}
fn changed_times_success(ctx: &mut TestContext, ft: FileType) {
    use nix::libc::makedev;

    let argument = match ft {
        FileType::Block => SFlag::S_IFBLK,
        FileType::Char => SFlag::S_IFCHR,
        _ => unreachable!(),
    };

    let file = ctx.gen_path();
    assert_times_changed()
        .path(ctx.base_path(), CTIME | MTIME)
        .paths(ctx.base_path(), &file, ATIME | CTIME | MTIME)
        .execute(ctx, false, || {
            assert!(mknod(
                &file,
                argument,
                Mode::from_bits_truncate(0o755),
                makedev(1, 2)
            )
            .is_ok());
        })
}

#[cfg(target_os = "illumos")]
crate::test_case! {
    /// mknod creates devices with old and new numbers
    create_old_new_device, root
}
#[cfg(target_os = "illumos")]
fn create_old_new_device(ctx: &mut TestContext) {
    {
        let file = ctx.gen_path();
        assert!(mknod(
            &file,
            argument,
            Mode::from_bits_truncate(mode),
            makedev(4095, 4095)
        )
        .is_ok());

        let stat = symlink_metadata(&file).unwrap();
        assert_eq!(stat.permissions().mode() as mode_t & ALLPERMS, mode);
        assert_eq!(major(stat.rdev()), major_num);
        assert_eq!(minor(stat.rdev()), minor_num);
        assert!(check(&stat.file_type()));

        let file = ctx.gen_path();

        assert_eq!(
            mknod(
                &file,
                argument,
                Mode::from_bits_truncate(mode),
                makedev(4096, 262144)
            ),
            Err(Errno::EINVAL)
        );
    }
}

// mknod/02.t
enametoolong_comp_test_case!(mknod(~path, SFlag::S_IFIFO, Mode::empty(), 0));

// mknod/03.t
enametoolong_path_test_case!(mknod(~path, SFlag::S_IFIFO, Mode::empty(), 0));

// mknod/04.t
enoent_comp_test_case!(mknod(~path, SFlag::S_IFIFO, Mode::empty(), 0));

// mknod/07.t
eloop_comp_test_case!(mknod(~path, SFlag::S_IFIFO, Mode::empty(), 0));
mod privileged {
    use super::*;

    fn mknod_block_wrapper(_: &mut TestContext, path: &Path) -> nix::Result<()> {
        mknod(path, SFlag::S_IFBLK, Mode::empty(), 0)
    }

    fn mknod_char_wrapper(_: &mut TestContext, path: &Path) -> nix::Result<()> {
        mknod(path, SFlag::S_IFCHR, Mode::empty(), 0)
    }

    // mknod/02.t
    enametoolong_comp_test_case!(mknod, mknod_block_wrapper, mknod_char_wrapper; root);
}
