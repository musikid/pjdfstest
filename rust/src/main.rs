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

use pjdfs_tests::{
    pjdfs_main,
    test::{ExclFeature, TestCase, TestContext, TEST_CASES},
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
        for feature in ExclFeature::iter() {
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

    let enabled_features: HashSet<ExclFeature> = config
        .features
        .keys()
        .filter_map(|k| k.as_str().try_into().ok())
        .collect();

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
        if let Some(sc) = &test_case.syscall {
            if !enabled_features.contains(sc) {
                println!(
                    "skipped {}: please add it to the configuration file if you want to run it",
                    sc
                );
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
