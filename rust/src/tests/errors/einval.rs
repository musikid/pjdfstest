use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    libc::off_t,
    sys::stat::Mode,
    unistd::{ftruncate, truncate},
};

use std::{fmt::Debug, path::Path};

use crate::{
    runner::context::{FileType, TestContext},
    utils::rename,
};

crate::test_case! {einval}
fn einval(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Regular).unwrap();

    // (f)truncate/13.t
    assert_eq!(truncate(&path, -1), Err(Errno::EINVAL));
    assert_eq!(truncate(&path, off_t::MIN), Err(Errno::EINVAL));
    let file = open(&path, OFlag::O_RDWR, Mode::empty()).unwrap();
    assert_eq!(ftruncate(file, -1), Err(Errno::EINVAL));
    let file = open(&path, OFlag::O_WRONLY, Mode::empty()).unwrap();
    assert_eq!(ftruncate(file, off_t::MIN), Err(Errno::EINVAL));

    // rename/19.t
    let dir = ctx.create(FileType::Dir).unwrap();
    let subdir = ctx.create_named(FileType::Dir, dir.join("subdir")).unwrap();
    assert!(matches!(
        rename(&subdir.join("."), &subdir.join("test")),
        Err(Errno::EINVAL | Errno::EBUSY)
    ));
    assert!(matches!(
        rename(&subdir.join(".."), &subdir.join("test")),
        Err(Errno::EINVAL | Errno::EBUSY)
    ));

    // rename/18.t
    let nested_subdir = ctx
        .create_named(FileType::Dir, subdir.join("nested"))
        .unwrap();
    assert_eq!(rename(&dir, &subdir), Err(Errno::EINVAL));
    assert_eq!(rename(&dir, &nested_subdir), Err(Errno::EINVAL));
}

crate::test_case! {
    /// open may return EINVAL when an attempt was made to open a descriptor
    /// with an illegal combination of O_RDONLY, O_WRONLY, and O_RDWR
    // open/23.t
    open_einval
}
fn open_einval(ctx: &mut TestContext) {
    fn assert_einval_open(ctx: &mut TestContext, flags: OFlag) {
        let path = ctx.create(FileType::Regular).unwrap();
        assert!(matches!(
            open(&path, flags, Mode::empty()),
            Ok(_) | Err(Errno::EINVAL)
        ));
    }

    assert_einval_open(ctx, OFlag::O_RDONLY | OFlag::O_RDWR);
    assert_einval_open(ctx, OFlag::O_WRONLY | OFlag::O_RDWR);
    assert_einval_open(ctx, OFlag::O_RDONLY | OFlag::O_WRONLY | OFlag::O_RDWR);
}
