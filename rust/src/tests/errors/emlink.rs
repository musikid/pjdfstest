use std::path::Path;

use nix::{errno::Errno, unistd::pathconf};

use crate::{
    config::Config,
    runner::context::{FileType, TestContext},
    utils::link,
};

const LINK_MAX_LIMIT: i64 = 65535;

// TODO: Some systems return bogus value, and testing directories might give different result than trying directly on the file
#[cfg(not(target_os = "linux"))]
fn has_reasonable_link_max(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    let link_max = pathconf(base_path, nix::unistd::PathconfVar::LINK_MAX)?
        .ok_or_else(|| anyhow::anyhow!("Failed to get LINK_MAX value"))?;

    if link_max >= LINK_MAX_LIMIT {
        anyhow::bail!(
            "LINK_MAX value is too high ({link_max}, expected smaller than {LINK_MAX_LIMIT})"
        );
    }

    Ok(())
}

#[cfg(target_os = "linux")]
// pathconf(_PC_LINK_MAX) on Linux returns 127 (LINUX_LINK_MAX) if the filesystem limit is unknown...
fn has_reasonable_link_max(_: &Config, base_path: &Path) -> anyhow::Result<()> {
    let link_max = pathconf(base_path, nix::unistd::PathconfVar::LINK_MAX)?
        .ok_or_else(|| anyhow::anyhow!("Failed to get LINK_MAX value"))?;

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

// POSIX states that open should return ELOOP, but FreeBSD returns EMLINK instead
#[cfg(target_os = "freebsd")]
crate::test_case! {
    /// open returns EMLINK when O_NOFOLLOW was specified and the target is a symbolic link
    open_nofollow
}
#[cfg(target_os = "freebsd")]
fn open_nofollow(ctx: &mut TestContext) {
    use nix::{
        fcntl::{open, OFlag},
        sys::stat::Mode,
    };

    let link = ctx.create(FileType::Symlink(None)).unwrap();

    assert_eq!(
        open(
            &link,
            OFlag::O_RDONLY | OFlag::O_CREAT | OFlag::O_NOFOLLOW,
            Mode::empty()
        ),
        Err(Errno::EMLINK)
    );
    assert_eq!(
        open(&link, OFlag::O_RDONLY | OFlag::O_NOFOLLOW, Mode::empty()),
        Err(Errno::EMLINK)
    );
    assert_eq!(
        open(&link, OFlag::O_WRONLY | OFlag::O_NOFOLLOW, Mode::empty()),
        Err(Errno::EMLINK)
    );
    assert_eq!(
        open(&link, OFlag::O_RDWR | OFlag::O_NOFOLLOW, Mode::empty()),
        Err(Errno::EMLINK)
    );
}
