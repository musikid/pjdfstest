use nix::unistd::chown;

use crate::{context::TestContext, utils::lchown};

use super::errors::eloop::{eloop_comp_test_case, eloop_final_comp_test_case};
use super::errors::enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case};
use super::errors::enoent::{
    enoent_comp_test_case, enoent_named_file_test_case, enoent_symlink_named_file_test_case,
};
use super::errors::enotdir::enotdir_comp_test_case;

fn chown_wrapper(ctx: &mut TestContext, path: &std::path::Path) -> nix::Result<()> {
    let user = ctx.get_new_user();
    chown(path, Some(user.uid), None)
}

enotdir_comp_test_case!(chown, chown_wrapper);

// chown/04.t
enoent_named_file_test_case!(chown, chown_wrapper);

// chown/04.t
enoent_comp_test_case!(chown, chown_wrapper);

// chown/04.t
enoent_symlink_named_file_test_case!(chown, chown_wrapper);

// chown/06.t
eloop_comp_test_case!(chown, chown_wrapper);

// chown/06.t
eloop_final_comp_test_case!(chown, chown_wrapper);

// chown/02.t
enametoolong_comp_test_case!(chown, chown_wrapper);

// chown/03.t
enametoolong_path_test_case!(chown, chown_wrapper);

mod lchown {
    use std::path::Path;

    use super::*;

    fn lchown_wrapper<P: AsRef<Path>>(ctx: &mut TestContext, path: P) -> nix::Result<()> {
        let path = path.as_ref();
        let user = ctx.get_new_user();
        lchown(path, Some(user.uid), Some(user.gid))
    }

    enotdir_comp_test_case!(lchown, lchown_wrapper);
    enoent_named_file_test_case!(lchown, lchown_wrapper);
    enoent_comp_test_case!(lchown, lchown_wrapper);

    // chown/06.t#L25
    eloop_comp_test_case!(lchown, lchown_wrapper);

    enametoolong_comp_test_case!(lchown, lchown_wrapper);
    enametoolong_path_test_case!(lchown, lchown_wrapper);
}
