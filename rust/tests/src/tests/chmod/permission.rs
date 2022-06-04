use crate::{
    pjdfs_test_case,
    test::{TestContext, TestResult},
};

// chmod/00.t:L58
fn test_ctime(ctx: &mut TestContext) -> TestResult {
    println!("testing!");

    Ok(())
}

pjdfs_test_case!(permission, test_ctime);
