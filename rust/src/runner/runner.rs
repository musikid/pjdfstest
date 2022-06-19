/// Macro which expands to a function which executes the tests.
#[macro_export]
macro_rules! pjdfs_main {
    ($( $test:path ),+) => {
        fn main() -> anyhow::Result<()> {
            for group in &[ $( $test ),+ ] {
                for test_case in group.test_cases.iter() {
                    for test in test_case.tests {
                        println!(
                            "{}\t",
                            format!("{}::{}::{}", group.name, test_case.name, test.name)
                        );
                        let mut context = TestContext::new();
                        (test.fun)(&mut context)?;
                    }
                }
            }

            Ok(())
        }
    };
}
