use std::io::{stdout, Write};

use pjdfs_tests::{pjdfs_main, test::TestContext, tests::chmod};

fn main() -> anyhow::Result<()> {
    for group in [chmod::tests] {
        for test_case in group.test_cases.iter() {
            for test in test_case.tests {
                print!(
                    "{}\t",
                    format!("{}::{}::{}", group.name, test_case.name, test.name)
                );
                stdout().lock().flush()?;
                let mut context = TestContext::new();
                match (test.fun)(&mut context) {
                    Ok(_) => println!("success"),
                    Err(e) => return Err(anyhow::anyhow!("{}", e)),
                }
            }
        }
    }

    Ok(())
}
