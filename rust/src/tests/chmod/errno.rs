use nix::{
    errno::Errno,
    sys::stat::{stat, Mode},
};

use crate::{runner::context::FileType, test::TestContext, utils::chmod};

#[cfg(target_os = "freebsd")]
use crate::test::{FileFlags, FileSystemFeature};

crate::test_case! {
    /// Returns ENOTDIR if a component of the path prefix is not a directory
    enotdir => [Regular, Fifo, Block, Char, Socket]
}
fn enotdir(ctx: &mut TestContext, f_type: FileType) {
    let not_dir = ctx.create(f_type).unwrap();
    let fake_path = not_dir.join("test");
    let res = chmod(&fake_path, Mode::from_bits_truncate(0o0644));
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), Errno::ENOTDIR);
}

crate::test_case! {
    /// chmod returns ENAMETOOLONG if a component of a pathname exceeded {NAME_MAX} characters
    enametoolong
}
fn enametoolong(ctx: &mut TestContext) {
    let path = ctx.create_max(FileType::Regular).unwrap();
    let expected_mode = 0o620;
    chmod(&path, Mode::from_bits_truncate(expected_mode)).unwrap();
    let actual_mode = stat(&path).unwrap().st_mode;
    assert_eq!(actual_mode & 0o777, expected_mode);

    let mut too_long_path = path;
    too_long_path.set_extension("x");
    let res = chmod(&too_long_path, Mode::from_bits_truncate(0o0620));
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), Errno::ENAMETOOLONG);
}

#[cfg(target_os = "freebsd")]
crate::test_case! {eperm_immutable_flag, FileSystemFeature::Chflags; FileFlags::SF_IMMUTABLE}
#[cfg(target_os = "freebsd")]
fn eperm_immutable_flag(ctx: &mut TestContext) {
    let _path = ctx.create(FileType::Regular).unwrap();
    //TODO: Complete
}
