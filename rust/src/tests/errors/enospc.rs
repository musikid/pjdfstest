use std::path::Path;

use anyhow::Result;
use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::{stat::Mode, statfs::statfs, statvfs::statvfs},
    unistd::{chown, mkdir, mkfifo, Uid},
};

use crate::{
    config::Config,
    runner::context::{FileType, SerializedTestContext},
    test::FileSystemFeature,
    utils::symlink,
};

const REMAINING_SPACE_LIMIT: i64 = 128 * 1024i64.pow(2);

/// Guard to check that the file system is small.
// TODO: Add a guard for mountpoint?
fn is_small(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    let stat = statfs(base_path)?;
    let available_blocks: i64 = stat.blocks_available().try_into()?;
    let frag_size: i64 = match stat.block_size().try_into()? {
        0 => anyhow::bail!("Cannot get file system fragment size"),
        num => num,
    };
    let remaining_space: i64 = available_blocks * frag_size;

    if remaining_space >= REMAINING_SPACE_LIMIT {
        anyhow::bail!("File system free space is too high")
    }

    Ok(())
}

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
