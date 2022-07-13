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

    #[options(help = "Match names exactly")]
    exact: bool,

    #[options(help = "Verbose mode")]
    verbose: bool,

    #[options(free, help = "Filter test names")]
    test_patterns: Vec<String>,
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
                .unwrap_or_else(|| Path::new("pjdfstest.toml")),
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

    let test_cases = inventory::iter::<TestCase>;
    let test_cases: Vec<_> = test_cases
        .into_iter()
        .filter(|case| {
            args.test_patterns.is_empty()
                || args.test_patterns.iter().any(|pat| {
                    if args.exact {
                        case.name == pat
                    } else {
                        case.name.contains(pat)
                    }
                })
        })
        .collect();

    let mut failed_tests_count: usize = 0;
    let mut succeeded_tests_count: usize = 0;
    let mut skipped_tests_count: usize = 0;

    for test_case in test_cases {
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
            message += features;
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

        print!("{}\t", test_case.name);

        if args.verbose {
            if !test_case.description.is_empty() {
                print!("\n\t{}\t\t", test_case.description);
            }
        }

        stdout().lock().flush()?;

        if should_skip {
            println!("skipped\n{}", message);
            skipped_tests_count += 1;
            continue;
        }

        let mut context = TestContext::new(&config.settings.naptime);
        //TODO: AssertUnwindSafe should be used with caution
        let mut ctx_wrapper = AssertUnwindSafe(&mut context);

        match catch_unwind(move || {
            (test_case.fun)(&mut ctx_wrapper);
        }) {
            Ok(_) => {
                println!("success");
                succeeded_tests_count += 1;
            }
            Err(e) => {
                let location = PANIC_LOCATION.get().unwrap();
                println!(
                    "error: {}, located in file {} at {}:{}",
                    e.downcast_ref::<String>()
                        .cloned()
                        .or_else(|| e.downcast_ref::<&str>().map(|&s| s.to_string()))
                        .unwrap_or_default(),
                    location.2,
                    location.0,
                    location.1
                );
                failed_tests_count += 1;
            }
        }
    }

    println!(
        "\nTests: {} failed, {} skipped, {} passed, {} total",
        failed_tests_count,
        skipped_tests_count,
        succeeded_tests_count,
        failed_tests_count + skipped_tests_count + succeeded_tests_count,
    );

    if failed_tests_count > 0 {
        Err(anyhow::anyhow!("Some tests have failed"))
    } else {
        Ok(())
    }
}
