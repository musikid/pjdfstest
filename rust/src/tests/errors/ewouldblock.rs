use nix::{
    fcntl::{open, OFlag},
    sys::stat::Mode,
};

use crate::runner::context::{FileType, TestContext};

#[cfg(target_os = "freebsd")]
crate::test_case! {
    /// open returns EWOULDBLOCK when O_NONBLOCK and one of O_SHLOCK or O_EXLOCK is specified and the file is locked
    open_lock
}
#[cfg(target_os = "freebsd")]
fn open_lock(ctx: &mut TestContext) {
    use nix::{errno::Errno, unistd::close};
    use std::path::Path;

    let file = ctx.create(FileType::Regular).unwrap();
    let fd = open(&file, OFlag::O_RDONLY | OFlag::O_SHLOCK, Mode::empty());
    assert!(fd.is_ok());
    let fd = fd.unwrap();
    assert!(open(
        &file,
        OFlag::O_RDONLY | OFlag::O_SHLOCK | OFlag::O_NONBLOCK,
        Mode::empty()
    )
    .and_then(close)
    .is_ok());
    close(fd).unwrap();

    // EWOULDBLOCK has the same value than EAGAIN on FreeBSD
    fn assert_ewouldblock(file: &Path, lockflag_locked: OFlag, lockflag_nonblock: OFlag) {
        let fd1 = open(file, OFlag::O_RDONLY | lockflag_locked, Mode::empty()).unwrap();
        assert!(matches!(
            open(
                file,
                OFlag::O_RDONLY | lockflag_nonblock | OFlag::O_NONBLOCK,
                Mode::empty()
            ),
            Err(Errno::EWOULDBLOCK)
        ));
        close(fd1).unwrap();
    }

    assert_ewouldblock(&file, OFlag::O_EXLOCK, OFlag::O_EXLOCK);
    assert_ewouldblock(&file, OFlag::O_SHLOCK, OFlag::O_EXLOCK);
    assert_ewouldblock(&file, OFlag::O_SHLOCK, OFlag::O_EXLOCK);
}
