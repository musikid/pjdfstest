use crate::config::Config;
use crate::flags::FileFlags;

use std::collections::HashSet;
use std::path::Path;

/// Macro to check whether any of the provided flags as an array is available in the configuration.
macro_rules! supports_any_flag {
    (@ $( $flag: expr ),+ $( , )*) => {
        supports_any_flag!(&[ $( $flag ),+ ])
    };

    ($flags: expr) => {
        |config, _p| $crate::tests::supports_any_flag_helper($flags, config, _p)
    }
}

pub(crate) use supports_any_flag;

/// Guard to check whether any of the provided flags is available in the configuration.
pub(crate) fn supports_any_flag_helper(
    flags: &[FileFlags],
    config: &Config,
    _: &Path,
) -> Result<(), anyhow::Error> {
    let flags: HashSet<_> = flags.iter().copied().collect();

    if config.features.file_flags.intersection(&flags).count() == 0 {
        anyhow::bail!("None of the flags used for this test are available in the configuration")
    }

    Ok(())
}

/// Guard to conditionally skip tests on platforms which do not support
/// all of the requested file flags.
macro_rules! supports_file_flags {
    ($($flags: ident),*) => {
        |config, _| {
            let flags = &[ $(crate::flags::FileFlags::$flags),* ].iter().copied().collect();
            if config.features.file_flags.is_superset(&flags) {
                Ok(())
            } else {
                let unsupported_flags = flags.difference(&config.features.file_flags)
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                anyhow::bail!("file flags {unsupported_flags} aren't supported")
            }
        }
    };
}

pub(crate) use supports_file_flags;

#[cfg(test)]
mod t {
    use crate::{config::Config, context::TestContext, test::TestCase};

    use super::*;

    crate::test_case! {support_flags_empty; supports_file_flags!()}
    fn support_flags_empty(_: &mut TestContext) {}
    #[test]
    fn support_flags_test_empty() {
        let config = Config::default();
        let tc: &TestCase = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::tests::t::support_flags_empty")
            .unwrap();
        assert_eq!(tc.guards.len(), 1);

        let guard = &tc.guards[0];
        assert!(guard(&config, Path::new("test")).is_ok());
    }

    crate::test_case! {support_flags_unique; supports_file_flags!(SF_APPEND)}
    fn support_flags_unique(_: &mut TestContext) {}
    #[test]
    fn support_flags_test_unique() {
        use std::collections::HashSet;

        use crate::flags::FileFlags;

        let mut config = Config::default();
        let tc: &TestCase = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::tests::t::support_flags_unique")
            .unwrap();
        assert_eq!(tc.guards.len(), 1);

        let guard = &tc.guards[0];
        assert!(guard(&config, Path::new("test")).is_err());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_ok());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND, FileFlags::UF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_ok());
    }

    crate::test_case! {support_flags_not_empty; supports_file_flags!(SF_APPEND, UF_APPEND)}
    fn support_flags_not_empty(_: &mut TestContext) {}
    #[test]
    fn support_flags_test_not_empty() {
        use std::collections::HashSet;

        use crate::flags::FileFlags;

        let mut config = Config::default();
        let tc: &TestCase = inventory::iter::<TestCase>()
            .find(|tc| tc.name == "pjdfstest::tests::t::support_flags_not_empty")
            .unwrap();
        assert_eq!(tc.guards.len(), 1);

        let guard = &tc.guards[0];
        assert!(guard(&config, Path::new("test")).is_err());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_err());

        config.features.file_flags = HashSet::from([FileFlags::SF_APPEND, FileFlags::UF_APPEND]);
        assert!(guard(&config, Path::new("test")).is_ok());

        config.features.file_flags = HashSet::from([
            FileFlags::SF_APPEND,
            FileFlags::UF_APPEND,
            FileFlags::SF_ARCHIVED,
        ]);
        assert!(guard(&config, Path::new("test")).is_ok());
    }
}
