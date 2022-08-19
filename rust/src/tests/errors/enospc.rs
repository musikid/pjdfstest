use std::{fs::File, io::Read, path::Path};

use anyhow::Result;
use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::{stat::Mode, statfs::statfs, statvfs::statvfs},
    unistd::{chown, mkdir, mkfifo, Uid},
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    utils::{is_small, link, symlink},
};

/// Saturate the free inodes.
fn saturate_inodes(ctx: &SerializedTestContext) -> Result<()> {
    // TODO: Switch to non-portable equivalent for more accurancy
    let stat = statvfs(ctx.base_path())?;

    let nfiles = stat.files_available();
    for _ in 0..nfiles {
        ctx.create(FileType::Regular)?;
    }

    Ok(())
}

/// Saturate free space.
fn saturate_space(ctx: &SerializedTestContext) -> Result<()> {
    // TODO: Switch to non-portable equivalent for more accurancy
    let mut file = File::create(ctx.gen_path())?;
    let stat = statvfs(ctx.base_path())?;
    let file_size = (stat.blocks_available() - 1) * stat.block_size() as u64;
    let mut zero = std::io::repeat(0).take(file_size);
    std::io::copy(&mut zero, &mut file)?;

    nix::unistd::sync();
    let stat = statvfs(ctx.base_path())?;
    debug_assert_eq!(stat.blocks_available(), 0);

    while let Ok(_) = ctx.create(FileType::Regular) {}

    Ok(())
}

/// Execute as unprivileged user.
macro_rules! as_unprivileged_user {
    ($ctx: ident, $fn:block) => {
        if Uid::effective().is_root() {
            let user = $ctx.get_new_user();
            chown($ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();
            $fn
        } else {
            $fn
        }
    };
}

crate::test_case! {
    /// Return ENOSPC if there are no free inodes on the file system
    no_inode, serialized; is_small
}
fn no_inode(ctx: &mut SerializedTestContext) {
    as_unprivileged_user!(ctx, {
        let file = ctx
            .create(crate::runner::context::FileType::Regular)
            .unwrap();
        let path = ctx.gen_path();
        saturate_inodes(ctx).unwrap();

        // mkdir/11.t
        assert_eq!(mkdir(&path, Mode::empty()), Err(Errno::ENOSPC));

        // mkfifo/11.t
        assert_eq!(mkfifo(&path, Mode::empty()), Err(Errno::ENOSPC));

        // open/19.t
        assert_eq!(
            open(&path, OFlag::O_CREAT | OFlag::O_RDONLY, Mode::empty()),
            Err(Errno::ENOSPC)
        );

        // symlink/11.t
        assert_eq!(symlink(&file, &path), Err(Errno::ENOSPC));
    });
}

crate::test_case! {
    /// link returns ENOSPC if the directory in which the entry for the new link is being placed
    /// cannot be extended because there is no space left on the file system containing the directory
    no_space, serialized; is_small
}
fn no_space(ctx: &mut SerializedTestContext) {
    as_unprivileged_user!(ctx, {
        let file = ctx
            .create(crate::runner::context::FileType::Regular)
            .unwrap();
        let path = ctx.gen_path();
        saturate_space(ctx).unwrap();

        // link/15.t
        assert_eq!(link(&file, &path), Err(Errno::ENOSPC));
    });
}
