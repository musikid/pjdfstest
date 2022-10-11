use nix::unistd::chown;

use crate::{runner::context::TestContext, utils::lchown};

use super::errors::enoent::{
    enoent_comp_test_case, enoent_named_file_test_case, enoent_symlink_named_file_test_case,
};
use super::errors::enotdir::enotdir_comp_test_case;

enotdir_comp_test_case!(chown, |ctx: &mut TestContext, path| {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
});

// chown/04.t
enoent_named_file_test_case!(chown, |ctx: &mut TestContext, path| {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
});

// chown/04.t
enoent_comp_test_case!(chown, |ctx: &mut TestContext, path| {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
});

// chown/04.t
enoent_symlink_named_file_test_case!(chown, |ctx: &mut TestContext, path| {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
});

mod lchown {
    use super::*;

    enotdir_comp_test_case!(lchown, |ctx: &mut TestContext, path| {
        let user = ctx.get_new_user();
        lchown(path, Some(user.uid), None)
    });

    enoent_named_file_test_case!(lchown, |ctx: &mut TestContext, path| {
        let user = ctx.get_new_user();
        lchown(path, Some(user.uid), None)
    });

    enoent_comp_test_case!(lchown, |ctx: &mut TestContext, path| {
        let user = ctx.get_new_user();
        lchown(path, Some(user.uid), None)
    });
}
