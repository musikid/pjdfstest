use std::path::Path;

use nix::{errno::Errno, sys::statfs::statfs, unistd::pathconf};

use crate::{
    config::Config,
    runner::context::{FileType, TestContext},
    utils::link,
};

const LINK_MAX_LIMIT: i64 = 65535;

// TODO: Some systems return bogus value, and testing directories can give different result than trying directly on the file
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
