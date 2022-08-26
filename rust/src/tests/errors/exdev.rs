use std::path::{Path, PathBuf};

use once_cell::sync::OnceCell;

use crate::{
    config::Config,
    runner::context::TestContext,
    utils::{link, rename},
};
use nix::errno::Errno;

// TODO: We don't want to give access to the config to tests,
// but is there a better way to do this?
static SECONDARY_FS: OnceCell<PathBuf> = OnceCell::new();

fn secondary_fs_available(config: &Config, _: &Path) -> anyhow::Result<()> {
    match &config.settings.secondary_fs {
        Some(path) => SECONDARY_FS.get_or_init(|| path.clone()),
        None => anyhow::bail!("No other file system has been defined"),
    };

    Ok(())
}

crate::test_case! {other_fs; secondary_fs_available}
fn other_fs(ctx: &mut TestContext) {
    let path = ctx
        .create(crate::runner::context::FileType::Regular)
        .unwrap();
    let other_fs_path = SECONDARY_FS.get().unwrap().join("file");

    assert_eq!(link(&path, &other_fs_path), Err(Errno::EXDEV));
    assert_eq!(rename(&path, &other_fs_path), Err(Errno::EXDEV));
}
