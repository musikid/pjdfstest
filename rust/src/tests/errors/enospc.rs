use std::path::Path;

use anyhow::Result;
use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::{stat::Mode, statvfs::statvfs},
    unistd::{chown, mkdir, mkfifo, Uid},
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    utils::{is_small, symlink},
};

/// Saturate the free inodes and leave one available.
fn saturate_inodes(ctx: &SerializedTestContext) -> Result<()> {
    // TODO: Switch to non-portable equivalent for more accurency
    let stat = statvfs(ctx.base_path())?;

    let nfiles = stat.files_available();
    for _ in 0..nfiles - 1 {
        ctx.create(FileType::Regular)?;
    }

    Ok(())
}

/// Execute as unprivileged user.
fn as_unprivileged_user<F>(ctx: &mut SerializedTestContext, mut f: F)
where
    F: FnMut(&Path, &Path),
{
    let path = ctx.gen_path();

    if Uid::effective().is_root() {
        let user = ctx.get_new_user();
        chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
        ctx.as_user(&user, None, || {
            let file = ctx
                .create(crate::runner::context::FileType::Regular)
                .unwrap();
            f(&file, &path)
        })
    } else {
        let file = ctx
            .create(crate::runner::context::FileType::Regular)
            .unwrap();
        f(&file, &path)
    }
}

crate::test_case! {
    /// Return ENOSPC if there are no free inodes on the file system on which the symbolic link is being created
    no_inode, serialized; is_small
}
fn no_inode(ctx: &mut SerializedTestContext) {
    saturate_inodes(ctx).unwrap();
    as_unprivileged_user(ctx, |file, path| {
        // mkdir/11.t
        assert_eq!(mkdir(path, Mode::empty()), Err(Errno::ENOSPC));

        // TODO: link

        // mkfifo/11.t
        assert_eq!(mkfifo(path, Mode::empty()), Err(Errno::ENOSPC));

        // open/19.t
        assert_eq!(
            open(path, OFlag::O_CREAT | OFlag::O_RDONLY, Mode::empty()),
            Err(Errno::ENOSPC)
        );

        // symlink/11.t
        assert_eq!(symlink(file, path), Err(Errno::ENOSPC));
    })
}
