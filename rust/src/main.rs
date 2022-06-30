use std::{
    io::{stdout, Write},
    panic::{catch_unwind, set_hook, AssertUnwindSafe, Location, PanicInfo},
};

use once_cell::sync::OnceCell;
use pjdfs_tests::{pjdfs_main, test::TestContext, tests::chmod};

struct PanicLocation(u32, u32, String);

static PANIC_LOCATION: OnceCell<PanicLocation> = OnceCell::new();

fn main() -> anyhow::Result<()> {
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

    for group in [chmod::tests] {
        for test_case in group.test_cases.iter() {
            for test in test_case.tests {
                print!(
                    "{}\t",
                    format!("{}::{}::{}", group.name, test_case.name, test.name)
                );
                stdout().lock().flush()?;
                let mut context = TestContext::new();
                //TODO: AssertUnwindSafe should be used with caution
                let mut ctx_wrapper = AssertUnwindSafe(&mut context);
                match catch_unwind(move || {
                    (test.fun)(&mut ctx_wrapper);
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
        }
    }

    Ok(())
}
