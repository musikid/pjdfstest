use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::Mode,
};

use crate::runner::context::{FileType, TestContext};

crate::test_case! {
    /// open returns ENXIO when O_NONBLOCK is set, the named file is a fifo, O_WRONLY is set, and no process has the file open for reading
    open_fifo_nonblock_wronly
}
fn open_fifo_nonblock_wronly(ctx: &mut TestContext) {
    let fifo = ctx.create(FileType::Fifo).unwrap();
    assert_eq!(
        open(&fifo, OFlag::O_WRONLY | OFlag::O_NONBLOCK, Mode::empty()),
        Err(Errno::ENXIO)
    );
}

// POSIX now states that returned error should be EOPNOTSUPP, but Linux still returns ENXIO
#[cfg(target_os = "linux")]
crate::test_case! {
    /// open returns ENXIO when trying to open UNIX domain socket
    open_socket
}
#[cfg(target_os = "linux")]
fn open_socket(ctx: &mut TestContext) {
    let socket = ctx.create(FileType::Socket).unwrap();

    assert_eq!(
        open(&socket, OFlag::O_RDONLY, Mode::empty()),
        Err(Errno::ENXIO)
    );
    assert_eq!(
        open(&socket, OFlag::O_WRONLY, Mode::empty()),
        Err(Errno::ENXIO)
    );
    assert_eq!(
        open(&socket, OFlag::O_RDWR, Mode::empty()),
        Err(Errno::ENXIO)
    );
}
