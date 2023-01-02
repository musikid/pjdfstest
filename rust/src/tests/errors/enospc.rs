use std::{fs::File, io::Read, path::Path};

use anyhow::Result;
use nix::sys::{statfs::statfs, statvfs::statvfs};

use crate::{
    config::Config,
    runner::context::{FileType, SerializedTestContext},
};

/// Guard to check that the file system is smaller than the fixed limit.
pub(crate) fn is_small(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    const REMAINING_SPACE_LIMIT: usize = 128 * 1024usize.pow(2);

    let stat = statfs(base_path)?;
    let available_blocks: usize = stat.blocks_available().try_into()?;
    let frag_size: usize = match stat.block_size().try_into()? {
        0 => anyhow::bail!("Cannot get file system fragment size"),
        num => num,
    };
    let remaining_space = available_blocks * frag_size;

    if remaining_space >= REMAINING_SPACE_LIMIT {
        anyhow::bail!("File system free space is too high")
    }

    Ok(())
}

/// Saturate the free inodes.
pub(crate) fn saturate_inodes(ctx: &SerializedTestContext) -> Result<()> {
    // TODO: Switch to non-portable equivalent for more accurancy
    let stat = statvfs(ctx.base_path())?;

    let nfiles = stat.files_available();
    for _ in 0..nfiles {
        ctx.create(FileType::Regular)?;
    }

    Ok(())
}

/// Saturate free space.
pub(crate) fn saturate_space(ctx: &SerializedTestContext) -> Result<()> {
    // TODO: Switch to non-portable equivalent for more accurancy
    let mut file = File::create(ctx.gen_path())?;
    let stat = statvfs(ctx.base_path())?;
    let file_size = (stat.blocks_available() - 1) * stat.block_size() as u64;
    let mut zero = std::io::repeat(0).take(file_size);
    std::io::copy(&mut zero, &mut file)?;

    nix::unistd::sync();

    while let Ok(_) = ctx.create(FileType::Regular) {}

    Ok(())
}

/// Create a test case which asserts that the sycall
/// returns ENOSPC if there are no free inodes on the file system.
/// There are multiple forms for this macro:
///
/// - A basic form which takes the syscall, and optionally a `~path` argument
///   to indicate where the `path` argument should be substituted if the path
///   is not the only argument taken by the syscall.
///
/// ```
/// // `unlink` accepts only a path as argument.
/// enospc_no_free_inodes_test_case!(unlink);
/// // `chflags` takes a path and the flags to set as arguments.
/// // We need to add `~path` where the path argument should normally be taken.
/// enospc_no_free_inodes_test_case!(chflags(~path, FileFlags::empty()));
/// ```
///
/// - A more complex form which takes multiple functions
///   with the context and the path as arguments for syscalls
///   requring to compute other arguments.
///
/// ```
/// enospc_no_free_inodes_test_case!(chown, |ctx: &mut TestContext, path: &Path| {
///   let user = ctx.get_new_user();
///   chown(path, Some(user.uid), None)
/// })
/// ````
macro_rules! enospc_no_free_inodes_test_case {
    ($syscall: ident, $($f: expr),+) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
                " returns ENOSPC if there are no free inodes on the file system")]
            enospc_no_free_inodes, serialized; $crate::tests::errors::enospc::is_small
        }
        fn enospc_no_free_inodes(ctx: &mut $crate::SerializedTestContext) {
            $crate::utils::as_unprivileged_user!(ctx, {
                $crate::tests::errors::enospc::saturate_inodes(ctx).unwrap();
                let path = ctx.gen_path();
                $( assert_eq!($f(ctx, &path), Err(nix::errno::Errno::ENOSPC)); )+
            });
        }
    };

    ($syscall: ident $( ($( $($before:expr),* ,)? ~path $(, $($after:expr),*)?) )?) => {
        enospc_no_free_inodes_test_case!($syscall, |_ctx: &crate::runner::context::TestContext,
                                             path: &std::path::Path| {
                $syscall($( $($($before),* ,)? )? path $( $(, $($after),*)? )?)
        });
    };
}

pub(crate) use enospc_no_free_inodes_test_case;
