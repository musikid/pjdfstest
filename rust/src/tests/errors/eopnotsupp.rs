use crate::runner::context::TestContext;

// POSIX now states that returned error should be EOPNOTSUPP, but Linux returns ENXIO
#[cfg(not(target_os = "linux"))]
crate::test_case! {
    /// open returns EOPNOTSUPP when trying to open UNIX domain socket
    open_socket
}
#[cfg(not(target_os = "linux"))]
fn open_socket(ctx: &mut TestContext) {
    use crate::runner::context::FileType;
    use nix::{
        errno::Errno,
        fcntl::{open, OFlag},
        sys::stat::Mode,
    };

    let socket = ctx.create(FileType::Socket).unwrap();

    assert_eq!(
        open(&socket, OFlag::O_RDONLY, Mode::empty()),
        Err(Errno::EOPNOTSUPP)
    );
    assert_eq!(
        open(&socket, OFlag::O_WRONLY, Mode::empty()),
        Err(Errno::EOPNOTSUPP)
    );
    assert_eq!(
        open(&socket, OFlag::O_RDWR, Mode::empty()),
        Err(Errno::EOPNOTSUPP)
    );
}
