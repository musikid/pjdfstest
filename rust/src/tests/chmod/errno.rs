use nix::{
    errno::Errno,
    sys::stat::{stat, Mode},
};

use crate::{runner::context::FileType, test::TestContext};

use super::chmod;

crate::test_case! {enotdir, root}
/// Returns ENOTDIR if a component of the path prefix is not a directory
fn enotdir(ctx: &mut TestContext) {
    for f_type in [
        FileType::Regular,
        FileType::Fifo,
        FileType::Block,
        FileType::Char,
        FileType::Socket,
    ] {
        let not_dir = ctx.create(f_type).unwrap();
        let fake_path = not_dir.join("test");
        let res = chmod(&fake_path, Mode::from_bits_truncate(0o0644));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Errno::ENOTDIR);
    }
}

crate::test_case! {enametoolong}
/// chmod returns ENAMETOOLONG if a component of a pathname exceeded {NAME_MAX} characters
fn enametoolong(ctx: &mut TestContext) {
    let path = ctx.create_max(FileType::Regular).unwrap();
    let expected_mode = 0o620;
    chmod(&path, Mode::from_bits_truncate(expected_mode)).unwrap();
    let actual_mode = stat(&path).unwrap().st_mode;
    assert_eq!(actual_mode & 0o777, expected_mode);

    let mut too_long_path = path.clone();
    too_long_path.set_extension("x");
    let res = chmod(&too_long_path, Mode::from_bits_truncate(0o0620));
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), Errno::ENAMETOOLONG);
}
