use std::{
    io::{stdout, Write},
    panic::{catch_unwind, set_hook, AssertUnwindSafe},
};

use pjdfs_tests::{pjdfs_main, test::TestContext, tests::chmod};

fn main() -> anyhow::Result<()> {
    //TODO: We should use panic info
    set_hook(Box::new(|ctx| {}));

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
                        if let Ok(e) = e.downcast::<String>() {
                            return Err(anyhow::anyhow!("{}", e));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
