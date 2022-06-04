use pjdfs_tests::{test::TestContext, tests::chmod, pjdfs_main};

fn main() {
    pjdfs_main!(chmod::tests);
}
