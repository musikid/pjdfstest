use nix::{
    errno::Errno,
    sys::stat::{lstat, Mode},
    unistd::pathconf,
    unistd::{chown, unlink},
};

use std::path::Path;

use super::{
    errors::{
        efault::efault_either_test_case,
        eloop::eloop_either_test_case,
        enametoolong::{enametoolong_either_comp_test_case, enametoolong_either_path_test_case},
        exdev::exdev_target_test_case,
    },
    CTIME, MTIME,
};

use crate::{
    config::Config,
    tests::errors::enospc::{is_small, saturate_space},
    utils::as_unprivileged_user,
};

use crate::{
    context::{FileType, SerializedTestContext, TestContext},
    tests::{
        assert_times_changed, assert_times_unchanged,
        errors::enoent::enoent_either_named_file_test_case,
        errors::enotdir::enotdir_comp_either_test_case, AsTimeInvariant,
    },
    utils::{chmod, link},
};

crate::test_case! {
    /// link creates hardlinks which share the same metadata
    // link/00.t#23-41
    share_metadata, root => [Regular, Fifo, Block, Char, Socket]
}
fn share_metadata(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let file_stat = lstat(&file).unwrap();
    assert_eq!(file_stat.st_nlink, 1);

    let first_link = ctx.gen_path();
    link(&file, &first_link).unwrap();
    let file_stat = lstat(&file).unwrap();
    let first_link_stat = lstat(&first_link).unwrap();
    assert_eq!(file_stat.st_nlink, 2);
    assert_eq!(file_stat.st_nlink, first_link_stat.st_nlink);
    assert_eq!(
        file_stat.as_time_invariant(),
        first_link_stat.as_time_invariant()
    );

    let second_link = ctx.gen_path();
    link(&first_link, &second_link).unwrap();

    let file_stat = lstat(&file).unwrap();
    let first_link_stat = lstat(&first_link).unwrap();
    let second_link_stat = lstat(&second_link).unwrap();
    assert_eq!(file_stat.st_nlink, 3);
    assert_eq!(file_stat.st_nlink, first_link_stat.st_nlink);
    assert_eq!(
        file_stat.as_time_invariant(),
        first_link_stat.as_time_invariant()
    );
    assert_eq!(
        first_link_stat.as_time_invariant(),
        second_link_stat.as_time_invariant()
    );

    chmod(&first_link, Mode::from_bits_truncate(0o201)).unwrap();
    let user = ctx.get_new_user();
    let group = ctx.get_new_group();
    chown(&first_link, Some(user.uid), Some(group.gid)).unwrap();

    let first_link_stat = lstat(&first_link).unwrap();
    let file_stat = lstat(&file).unwrap();
    let second_link_stat = lstat(&second_link).unwrap();
    assert_eq!(
        file_stat.as_time_invariant(),
        first_link_stat.as_time_invariant()
    );
    assert_eq!(
        first_link_stat.as_time_invariant(),
        second_link_stat.as_time_invariant()
    );
}

crate::test_case! {
    /// Removing a link should only change the number of links
    // link/00.t
    remove_link => [Regular, Fifo, Block, Char, Socket]
}
fn remove_link(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let first_link = ctx.gen_path();
    let second_link = ctx.gen_path();

    link(&file, &first_link).unwrap();
    link(&first_link, &second_link).unwrap();

    unlink(&file).unwrap();
    assert!(!file.exists());

    let first_link_stat = lstat(&first_link).unwrap();
    let second_link_stat = lstat(&second_link).unwrap();
    assert_eq!(first_link_stat.st_nlink, 2);
    assert_eq!(
        first_link_stat.as_time_invariant(),
        second_link_stat.as_time_invariant()
    );

    unlink(&second_link).unwrap();
    assert!(!second_link.exists());

    let first_link_stat = lstat(&first_link).unwrap();
    assert_eq!(first_link_stat.st_nlink, 1);

    unlink(&first_link).unwrap();
    assert!(!first_link.exists());
}

crate::test_case! {
    /// link changes ctime of file along with ctime and mtime of parent when sucessful
    // link/00.t
    changed_ctime_success => [Regular, Fifo, Block, Char, Socket]
}
fn changed_ctime_success(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let new_path = ctx.gen_path();

    assert_times_changed()
        .path(&file, CTIME)
        .path(ctx.base_path(), CTIME | MTIME)
        .execute(ctx, false, || {
            assert!(link(&file, &new_path).is_ok());
        });
}
crate::test_case! {
    /// link changes neither ctime of file nor ctime or mtime of parent when it fails
    // link/00.t#77
    unchanged_ctime_fails, serialized, root => [Regular, Fifo, Block, Char, Socket]
}
fn unchanged_ctime_fails(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let new_path = ctx.gen_path();

    let user = ctx.get_new_user();
    assert_times_unchanged()
        .path(&file, CTIME)
        .path(ctx.base_path(), CTIME | MTIME)
        .execute(ctx, false, || {
            ctx.as_user(user, None, || {
                assert!(matches!(
                    link(&file, &new_path),
                    Err(Errno::EPERM | Errno::EACCES)
                ));
            })
        });
}

// link/01.t
enotdir_comp_either_test_case!(link);

const LINK_MAX_LIMIT: i64 = 65535;

// BUG: Some systems return bogus value, and testing directories
// might give different result than trying directly on the file
fn has_reasonable_link_max(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    let link_max = pathconf(base_path, nix::unistd::PathconfVar::LINK_MAX)?
        .ok_or_else(|| anyhow::anyhow!("Failed to get LINK_MAX value"))?;

    // pathconf(_PC_LINK_MAX) on Linux returns 127 (LINUX_LINK_MAX) if the filesystem limit is unknown...
    #[cfg(target_os = "linux")]
    if link_max == 127 {
        anyhow::bail!("Cannot get value for LINK_MAX: filesystem limit is unknown");
    }

    if link_max >= LINK_MAX_LIMIT {
        anyhow::bail!(
            "LINK_MAX value is too high ({link_max}, expected smaller than {LINK_MAX_LIMIT})"
        );
    }

    Ok(())
}

crate::test_case! {
    /// link returns EMLINK if the link count of the file named by name1 would exceed {LINK_MAX}
    link_count_max; has_reasonable_link_max
}
fn link_count_max(ctx: &mut TestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    let link_max = pathconf(&file, nix::unistd::PathconfVar::LINK_MAX)
        .unwrap()
        .unwrap();

    for _ in 0..link_max - 1 {
        link(&file, &ctx.gen_path()).unwrap();
    }

    assert_eq!(link(&file, &ctx.gen_path()), Err(Errno::EMLINK));
}

// link/02.t
enametoolong_either_comp_test_case!(link);

// link/03.t
enametoolong_either_path_test_case!(link);

// link/04.t
enoent_either_named_file_test_case!(link);

// link/08.t
eloop_either_test_case!(link);

// link/09.t
crate::test_case! {
    /// link returns ENOENT if the source file does not exist
    enoent_source_not_exists
}
fn enoent_source_not_exists(ctx: &mut TestContext) {
    let source = ctx.gen_path();
    let dest = ctx.gen_path();

    assert_eq!(link(&source, &dest), Err(Errno::ENOENT));
}

crate::test_case! {
    /// link returns EEXIST if the destination file exists
    eexist_dest_exists => [Regular, Dir, Fifo, Block, Char, Socket, Symlink(None)]
}
fn eexist_dest_exists(ctx: &mut TestContext, ft: FileType) {
    let path = ctx.create(ft).unwrap();
    let regular_file = ctx.create(FileType::Regular).unwrap();

    assert_eq!(link(&regular_file, &path), Err(nix::errno::Errno::EEXIST));
}

crate::test_case! {
    /// link returns ENOSPC if the directory in which the entry for the new link is being placed
    /// cannot be extended because there is no space left on the file system containing the directory
    // link/15.t
    enospc_no_space, serialized; is_small
}
fn enospc_no_space(ctx: &mut SerializedTestContext) {
    as_unprivileged_user!(ctx, {
        let file = ctx.create(FileType::Regular).unwrap();
        let path = ctx.gen_path();
        saturate_space(ctx).unwrap();

        assert_eq!(link(&file, &path), Err(Errno::ENOSPC));
    });
}

// link/14.t
exdev_target_test_case!(link);

// link/17.t
efault_either_test_case!(link, nix::libc::link);
