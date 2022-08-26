use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::Mode,
    unistd::truncate,
};

use crate::{
    runner::context::{FileType, TestContext},
    utils::rename,
};

crate::test_case! {
    /// Return EISDIR if the named file is a directory
    eisdir
}
fn eisdir(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Dir).unwrap();

    // open/13.t
    assert_eq!(
        open(&path, OFlag::O_WRONLY, Mode::empty()),
        Err(Errno::EISDIR)
    );
    assert_eq!(
        open(&path, OFlag::O_RDWR, Mode::empty()),
        Err(Errno::EISDIR)
    );
    assert_eq!(
        open(&path, OFlag::O_RDONLY | OFlag::O_TRUNC, Mode::empty()),
        Err(Errno::EISDIR)
    );
    assert_eq!(
        open(&path, OFlag::O_WRONLY | OFlag::O_TRUNC, Mode::empty()),
        Err(Errno::EISDIR)
    );
    assert_eq!(
        open(&path, OFlag::O_RDWR | OFlag::O_TRUNC, Mode::empty()),
        Err(Errno::EISDIR)
    );

    // (f)truncate/09.t
    assert_eq!(truncate(&path, 0), Err(Errno::EISDIR));
}

crate::test_case! {
    // rename/14.t
    eisdir_rename => [Regular, Fifo, Block, Char, Socket, Symlink(None)]
}
fn eisdir_rename(ctx: &mut TestContext, ft: FileType) {
    let dir = ctx.create(FileType::Dir).unwrap();
    let not_dir_file = ctx.create(ft).unwrap();
    assert_eq!(rename(&not_dir_file, &dir), Err(Errno::EISDIR));
}
