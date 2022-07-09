use std::{
    collections::HashSet,
    io::{stdout, Write},
    panic::{catch_unwind, set_hook, AssertUnwindSafe},
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

use pjdfs_tests::test::{FileSystemFeature, TestCase, TestContext};

mod config;

struct PanicLocation(u32, u32, String);

static PANIC_LOCATION: OnceCell<PanicLocation> = OnceCell::new();

#[derive(Debug, Options)]
struct ArgOptions {
    #[options(help = "print help message")]
    help: bool,

    #[options(help = "Path of the configuration file")]
    configuration_file: Option<PathBuf>,

    #[options(help = "List opt-in features")]
    list_features: bool,
}

fn main() -> anyhow::Result<()> {
    let args = ArgOptions::parse_args_default_or_exit();

    if args.list_features {
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

    let enabled_features: HashSet<_> = config.features.fs_features.keys().into_iter().collect();

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

    let enabled_flags: HashSet<_> = config.features.file_flags.iter().collect();

    for test_case in inventory::iter::<TestCase> {
        //TODO: There's probably a better way to do this...
        let mut should_skip = false;

        let mut message = String::new();

        let features: HashSet<_> = test_case.required_features.iter().collect();
        let missing_features: Vec<_> = features.difference(&enabled_features).collect();
        if !missing_features.is_empty() {
            should_skip = true;

            let features = &missing_features
                .iter()
                .map(|feature| format!("{}", feature))
                .collect::<Vec<_>>()
                .join(", ");

            message += "requires features: ";
            message += &features;
            message += "\n";
        }

        let required_flags: HashSet<_> = test_case.required_file_flags.iter().collect();
        let missing_flags: Vec<_> = required_flags.difference(&enabled_flags).collect();
        if !missing_flags.is_empty() {
            should_skip = true;

            let flags = missing_flags
                .iter()
                .map(|f| {
                    let f = f.to_string();

                    ["\"", &f, "\""].join("")
                })
                .collect::<Vec<_>>()
                .join(", ");

            message += "requires flags: ";
            message += &flags;
            message += "\n";
        }

        if should_skip {
            println!("skipped '{}'\n{}", test_case.name, message);
            continue;
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
