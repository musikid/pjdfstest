use std::{fmt::Debug, fs::File, path::Path, process::Command};

use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::Mode,
    unistd::truncate,
};

use crate::{runner::context::TestContext, utils::chmod};

crate::test_case! {
    /// Return ETXTBSY when the file is a pure procedure (shared text) file that is being executed
    etxtbsy
}
fn etxtbsy(ctx: &mut TestContext) {
    /// Asserts that it returns ETXTBSY when the file is a pure procedure (shared text) file that is being executed.
    // TODO: Refactor this?
    fn assert_etxtbsy<F, T: Debug>(ctx: &mut TestContext, f: F)
    where
        F: Fn(&Path) -> nix::Result<T>,
    {
        let sleep_path =
            String::from_utf8(Command::new("which").arg("sleep").output().unwrap().stdout).unwrap();
        let sleep_path = sleep_path.trim();

        let exec_path = ctx.base_path().join("sleep");
        std::io::copy(
            &mut File::open(sleep_path).unwrap(),
            &mut File::create(&exec_path).unwrap(),
        )
        .unwrap();

        chmod(&exec_path, Mode::from_bits_truncate(0o755)).unwrap();

        let mut sleep_process = Command::new(&exec_path).arg("5").spawn().unwrap();
        assert_eq!(f(&exec_path).unwrap_err(), Errno::ETXTBSY);

        sleep_process.kill().unwrap();
    }

    // open/20.t
    assert_etxtbsy(ctx, |p| open(p, OFlag::O_WRONLY, Mode::empty()));
    assert_etxtbsy(ctx, |p| open(p, OFlag::O_RDWR, Mode::empty()));
    assert_etxtbsy(ctx, |p| {
        open(p, OFlag::O_RDONLY | OFlag::O_TRUNC, Mode::empty())
    });
    // (f)truncate/11.t
    assert_etxtbsy(ctx, |p| truncate(p, 123));
}
