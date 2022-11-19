use nix::{
    errno::Errno,
    sys::stat::{stat, Mode},
};

use crate::{context::FileType, test::TestContext, utils::chmod};

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
    let path = ctx.create_name_max(FileType::Regular).unwrap();
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
