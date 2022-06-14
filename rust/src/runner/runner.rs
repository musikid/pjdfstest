/// Macro which expands to a function which executes the tests.
#[macro_export]
macro_rules! pjdfs_main {
    ($( $test:path ),+) => {
        for group in &[ $( $test ),+ ] {
            for test_case in group.test_cases.iter() {
                for test in test_case.tests {
                    print!(
                        "{}\t",
                        format!("{}::{}::{}", group.name, test_case.name, test.name)
                    );
                    let mut context = TestContext::new();
                    match (test.fun)(&mut context) {
                        Ok(_) => {
                            println!("success");
                        }
                        Err(e) => {
                            println!("error: {}", e);
                        }
                    }
                }
            }
        }
    };
}
