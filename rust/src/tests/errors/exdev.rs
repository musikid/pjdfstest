use std::path::{Path};



use crate::{
    config::Config,
};


/// Guard which checks if a secondary file system has been configured.
pub(crate) fn secondary_fs_available(config: &Config, _: &Path) -> anyhow::Result<()> {
    config
        .features
        .secondary_fs
        .is_some()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("No secondary file-system has been configured."))
}

/// Create a test-case for a syscall which returns `EXDEV` when the target is on a different file-system.
/// The test-case will be skipped if no secondary file system has been configured.
///
/// ```rust,ignore
/// exdev_target_test_case!(link);
/// ```
macro_rules! exdev_target_test_case {
    ($syscall: ident) => {
        crate::test_case! {
            #[doc = concat!(stringify!($syscall),
            " returns EXDEV when the target is on a different file-system")]
            exdev_target; crate::tests::errors::exdev::secondary_fs_available
        }
        fn exdev_target(ctx: &mut crate::TestContext) {
            let path = ctx.create(crate::context::FileType::Regular).unwrap();
            let other_fs_path = ctx
                .features_config()
                .secondary_fs
                .as_ref()
                .unwrap()
                .join("file");

            assert_eq!($syscall(&path, &other_fs_path), Err(Errno::EXDEV));
        }
    };
}

pub(crate) use exdev_target_test_case;