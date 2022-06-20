use nix::{
    errno::Errno,
    sys::stat::{stat, Mode},
};

use crate::{
    pjdfs_test_case,
    runner::context::FileType,
    test::{TestContext, TestResult},
    test_assert, test_assert_eq,
};

use super::chmod;

pjdfs_test_case!(errno, { test: test_enotdir, require_root: true }, { test: test_enametoolong });

/// Returns ENOTDIR if a component of the path prefix is not a directory
fn test_enotdir(ctx: &mut TestContext) -> TestResult {
    for f_type in [
        FileType::Regular,
        FileType::Fifo,
        FileType::Block,
        FileType::Char,
        FileType::Socket,
    ] {
        let not_dir = ctx.create(f_type)?;
        let fake_path = not_dir.join("test");
        let res = chmod(&fake_path, Mode::from_bits_truncate(0o0644));
        test_assert!(res.is_err());
        test_assert_eq!(res.unwrap_err(), Errno::ENOTDIR);
    }

    Ok(())
}

/// chmod returns ENAMETOOLONG if a component of a pathname exceeded {NAME_MAX} characters
fn test_enametoolong(ctx: &mut TestContext) -> TestResult {
    let path = ctx.create_max(FileType::Regular)?;
    let expected_mode = 0o620;
    chmod(&path, Mode::from_bits_truncate(expected_mode))?;
    let actual_mode = stat(&path)?.st_mode;
    test_assert_eq!(actual_mode & 0o777, expected_mode);

    let mut too_long_path = path.clone();
    too_long_path.set_extension("x");
    let res = chmod(&too_long_path, Mode::from_bits_truncate(0o0620));
    test_assert!(res.is_err());
    test_assert_eq!(res.unwrap_err(), Errno::ENAMETOOLONG);

    Ok(())
}
