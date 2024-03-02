// https://github.com/rust-lang/rust-clippy/issues/1553
#![allow(clippy::redundant_closure_call)]

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    collections::HashSet,
    env::current_dir,
    io::{stdout, Write},
    panic::{catch_unwind, set_hook},
    path::PathBuf,
    sync::Mutex
};

use config::Config;
use figment::{
    providers::{Format, Serialized, Toml},
    Figment,
};
use gumdrop::Options;
use nix::{
    sys::stat::{umask, Mode},
    unistd::Uid,
};
use strum::{EnumMessage, IntoEnumIterator};

use tempfile::{tempdir_in, TempDir};

mod config;
mod context;
mod features;
mod flags;
mod macros;
mod test;
mod tests;
mod utils;

use test::{FileSystemFeature, SerializedTestContext, TestCase, TestContext, TestFn};

use crate::utils::chmod;

static BACKTRACE: Mutex<Option<Backtrace>> = Mutex::new(None);

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

    #[options(help = "Path where the test suite will be executed")]
    path: Option<PathBuf>,

    #[options(free, help = "Filter test names")]
    test_patterns: Vec<String>,

    #[options(help = "Path to a secondary file system")]
    secondary_fs: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let args = ArgOptions::parse_args_default_or_exit();

    if args.list_features {
        for feature in FileSystemFeature::iter() {
            println!("{feature}: {}", feature.get_documentation().unwrap());
        }
        return Ok(());
    }

    let config: Config = {
        let mut figment = Figment::from(Serialized::defaults(Config::default()));
        if let Some(path) = args.configuration_file.as_deref() {
            figment = figment.merge(Toml::file(path))
        }

        let mut config: Config = figment.extract()?;
        config.features.secondary_fs = args.secondary_fs;
        config
    };

    let path = args
        .path
        .ok_or_else(|| anyhow::anyhow!("cannot get current dir"))
        .or_else(|_| current_dir())?;
    let base_dir = tempdir_in(path)?;

    set_hook(Box::new(|_| {
        *BACKTRACE.lock().unwrap() = Some(Backtrace::capture());
    }));

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

    umask(Mode::empty());

    let (failed_count, skipped_count, success_count) =
        run_test_cases(&test_cases, args.verbose, &config, base_dir)?;

    println!(
        "\nTests: {} failed, {} skipped, {} passed, {} total",
        failed_count,
        skipped_count,
        success_count,
        failed_count + skipped_count + success_count,
    );

    if failed_count > 0 {
        Err(anyhow::anyhow!("Some tests have failed"))
    } else {
        Ok(())
    }
}

/// Run provided test cases and filter according to features and flags availability.
//TODO: Refactor this function
fn run_test_cases(
    test_cases: &[&TestCase],
    verbose: bool,
    config: &Config,
    base_dir: TempDir,
) -> Result<(usize, usize, usize), anyhow::Error> {
    let mut failed_tests_count: usize = 0;
    let mut succeeded_tests_count: usize = 0;
    let mut skipped_tests_count: usize = 0;

    let is_root = Uid::current().is_root();

    let enabled_features: HashSet<_> = config.features.fs_features.keys().collect();

    let entries = &config.dummy_auth.entries;

    for test_case in test_cases {
        //TODO: There's probably a better way to do this...
        let mut should_skip = test_case.require_root && !is_root;
        let mut skip_message = String::new();

        if should_skip {
            skip_message += "requires root privileges\n";
        }

        let features: HashSet<_> = test_case.required_features.iter().collect();
        let missing_features: Vec<_> = features.difference(&enabled_features).collect();
        if !missing_features.is_empty() {
            should_skip = true;

            let features = &missing_features
                .iter()
                .map(|feature| format!("{}", feature))
                .collect::<Vec<_>>()
                .join(", ");

            skip_message += "requires features: ";
            skip_message += features;
            skip_message += "\n";
        }

        let temp_dir = tempdir_in(base_dir.path()).unwrap();
        // FIX: some tests need a 0o755 base dir
        chmod(temp_dir.path(), Mode::from_bits_truncate(0o755)).unwrap();

        if test_case
            .guards
            .iter()
            .any(|guard| guard(config, temp_dir.path()).is_err())
        {
            should_skip = true;
            skip_message += &*test_case
                .guards
                .iter()
                .filter_map(|guard| guard(config, base_dir.path()).err())
                .map(|err| err.to_string())
                .collect::<Vec<String>>()
                .join("\n");
            skip_message += "\n";
        }

        print!("{}\t", test_case.name);

        if verbose && !test_case.description.is_empty() {
            print!("\n\t{}\t\t", test_case.description);
        }

        stdout().lock().flush()?;

        if should_skip {
            println!("skipped\n{}", skip_message);
            skipped_tests_count += 1;
            continue;
        }

        let result = catch_unwind(|| match test_case.fun {
            TestFn::NonSerialized(fun) => {
                let mut context = TestContext::new(config, entries, temp_dir.path());

                (fun)(&mut context)
            }
            TestFn::Serialized(fun) => {
                let mut context = SerializedTestContext::new(config, entries, temp_dir.path());

                (fun)(&mut context)
            }
        });

        match result {
            Ok(_) => {
                println!("success");
                succeeded_tests_count += 1;
            }
            Err(e) => {
                let backtrace = BACKTRACE.lock().unwrap()
                    .take()
                    .filter(|bt| bt.status() == BacktraceStatus::Captured);
                let panic_information = match e.downcast::<String>() {
                    Ok(v) => *v,
                    Err(e) => match e.downcast::<&str>() {
                        Ok(v) => v.to_string(),
                        _ => "Unknown Source of Error".to_owned()
                    }
                };
                if let Some(backtrace) = backtrace {
                    println!("{}\nBacktrace:\n{}", panic_information, backtrace);
                } else {
                    println!("{}", panic_information);
                }
                failed_tests_count += 1;
            }
        }
    }

    Ok((
        failed_tests_count,
        skipped_tests_count,
        succeeded_tests_count,
    ))
}
