use nix::unistd::chown;

use crate::{runner::context::TestContext, utils::lchown};

use super::errors::enotdir::assert_enotdir_comp;

assert_enotdir_comp!(chown, |ctx: &mut TestContext, path| {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
});

mod lchown {
    use super::*;

    assert_enotdir_comp!(lchown, |ctx: &mut TestContext, path| {
        let user = ctx.get_new_user();
        lchown(path, Some(user.uid), None)
    });
}
