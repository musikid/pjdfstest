use std::{
    collections::HashSet,
    io::{stdout, Write},
    panic::{catch_unwind, set_hook, AssertUnwindSafe, Location, PanicInfo},
    path::{Path, PathBuf},
};

use config::Config;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use gumdrop::Options;
use once_cell::sync::OnceCell;
use strum::IntoEnumIterator;

#[cfg(any(
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "macos",
    target_os = "ios",
    target_os = "watchos",
))]
use pjdfs_tests::test::FileFlags;
use pjdfs_tests::{
    pjdfs_main,
    test::{FileSystemFeature, TestCase, TestContext, TEST_CASES},
};
mod config;

struct PanicLocation(u32, u32, String);

static PANIC_LOCATION: OnceCell<PanicLocation> = OnceCell::new();

#[derive(Debug, Options)]
struct ArgOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(help = "Path of the configuration file")]
    configuration_file: Option<PathBuf>,

    #[options(help = "List opt-in syscalls")]
    list_syscalls: bool,
}

fn main() -> anyhow::Result<()> {
    let args = ArgOptions::parse_args_default_or_exit();

    if args.list_syscalls {
        for feature in FileSystemFeature::iter() {
            println!("{}", feature);
        }
        return Ok(());
    }

    let config: Config = Figment::new()
        .merge(Toml::file(
            args.configuration_file
                .as_deref()
                .unwrap_or(Path::new("pjdfstest.toml")),
        ))
        .extract()?;

    let mut enabled_features: Vec<FileSystemFeature> =
        config.features.fs_features.keys().cloned().collect();

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    if let Some(flags) = config.features.file_flags {
        let flags: Vec<_> = flags.iter().cloned().collect();
        let flags = Box::new(flags);
        // TODO: It's not going to change for the program lifetime, but is there a better alternative than leaking?
        enabled_features.push(FileSystemFeature::FileFlags(Box::leak(flags)));
    }

    let enabled_features: HashSet<_> = enabled_features.into_iter().collect();

    set_hook(Box::new(|ctx| {
        if let Some(location) = ctx.location() {
            let _ = PANIC_LOCATION.set(PanicLocation(
                location.line(),
                location.column(),
                location.file().into(),
            ));
        } else {
            unimplemented!()
        }
    }));

    for test_case in TEST_CASES {
        if let Some(features) = &test_case.required_features {
            let features = features.iter().cloned().collect::<HashSet<_>>();
            let missing_features = features.difference(&enabled_features);
            if missing_features.clone().count() > 0 {
                println!(
                    "skipped {}, please add the following features to your configuration:",
                    test_case.name
                );
                for feature in missing_features {
                    println!(
                        "{}\n",
                        match feature {
                            #[cfg(any(
                                target_os = "openbsd",
                                target_os = "netbsd",
                                target_os = "freebsd",
                                target_os = "dragonfly",
                                target_os = "macos",
                                target_os = "ios",
                                target_os = "watchos",
                            ))]
                            FileSystemFeature::FileFlags(flags) => {
                                let flags = flags
                                    .iter()
                                    .map(|f| format!(r#""{}""#, f))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                format!("[features]\nfile_flags = [{}]", flags)
                            }
                            _ => format!("[features.{}]", feature.to_string()),
                        }
                    );
                }
                continue;
            }
        }

        print!("{}\t", test_case.name);
        stdout().lock().flush()?;
        let mut context = TestContext::new();
        //TODO: AssertUnwindSafe should be used with caution
        let mut ctx_wrapper = AssertUnwindSafe(&mut context);
        match catch_unwind(move || {
            (test_case.fun)(&mut ctx_wrapper);
        }) {
            Ok(_) => println!("success"),
            Err(e) => {
                let location = PANIC_LOCATION.get().unwrap();
                anyhow::bail!(
                    "{}
                    Located in file {} at {}:{}
                    ",
                    e.downcast_ref::<String>()
                        .cloned()
                        .or_else(|| e.downcast_ref::<&str>().map(|&s| s.to_string()))
                        .unwrap_or_default(),
                    location.2,
                    location.0,
                    location.1
                )
            }
        }
    }

    Ok(())
}
