use nix::errno::Errno;
use nix::unistd::chown;

use crate::context::{FileType, SerializedTestContext, TestContext};
use crate::utils::lchown;

use super::errors::efault::efault_path_test_case;
use super::errors::eloop::{eloop_comp_test_case, eloop_final_comp_test_case};
use super::errors::enametoolong::{enametoolong_comp_test_case, enametoolong_path_test_case};
use super::errors::enoent::{
    enoent_comp_test_case, enoent_named_file_test_case, enoent_symlink_named_file_test_case,
};
use super::errors::enotdir::enotdir_comp_test_case;
use super::errors::erofs::erofs_named_test_case;

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

// chown/09.t
erofs_named_test_case!(chown, chown_wrapper);

// chown/10.t
efault_path_test_case!(chown, |ptr| nix::libc::chown(ptr, 0, 0));

crate::test_case! {
    /// chown returns EPERM if the operation would change the ownership, but the effective user ID is not the super-user and the process is not an owner of the file
    // chown/07.t
    euid_not_root_not_owner, serialized, root => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn euid_not_root_not_owner(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();
    chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();

    let file = ctx.create(ft).unwrap();
    chown(&file, Some(user.uid), Some(user.gid)).unwrap();

    let another_user = ctx.get_new_user();

    ctx.as_user(user, None, || {
        assert_eq!(
            chown(&file, Some(another_user.uid), Some(another_user.gid)),
            Err(Errno::EPERM)
        );
    });
    ctx.as_user(another_user, None, || {
        assert_eq!(
            chown(&file, Some(user.uid), Some(user.gid)),
            Err(Errno::EPERM)
        );
    });
    ctx.as_user(another_user, None, || {
        assert_eq!(
            chown(&file, Some(another_user.uid), Some(another_user.gid)),
            Err(Errno::EPERM)
        );
    });
    ctx.as_user(user, None, || {
        assert_eq!(
            chown(&file, None, Some(another_user.gid)),
            Err(Errno::EPERM)
        );
    });

    let link = ctx.create(FileType::Symlink(Some(file))).unwrap();

    ctx.as_user(user, None, || {
        assert_eq!(
            chown(&link, Some(another_user.uid), Some(another_user.gid)),
            Err(Errno::EPERM)
        );
    });
    ctx.as_user(another_user, None, || {
        assert_eq!(
            chown(&link, Some(user.uid), Some(user.gid)),
            Err(Errno::EPERM)
        );
    });
    ctx.as_user(another_user, None, || {
        assert_eq!(
            chown(&link, Some(another_user.uid), Some(another_user.gid)),
            Err(Errno::EPERM)
        );
    });
    ctx.as_user(user, None, || {
        assert_eq!(
            chown(&link, None, Some(another_user.gid)),
            Err(Errno::EPERM)
        );
    });
}

mod lchown {
    use std::path::Path;

    use super::*;

    crate::test_case! {
        /// chown returns EPERM if the operation would change the ownership, but the effective user ID is not the super-user and the process is not an owner of the file
        // chown/07.t
        euid_not_root_not_owner, serialized, root
    }
    fn euid_not_root_not_owner(ctx: &mut SerializedTestContext) {
        let user = ctx.get_new_user();
        chown(ctx.base_path(), Some(user.uid), Some(user.gid)).unwrap();

        let file = ctx.create(FileType::Symlink(None)).unwrap();

        let another_user = ctx.get_new_user();

        ctx.as_user(user, None, || {
            assert_eq!(
                lchown(&file, Some(another_user.uid), Some(another_user.gid)),
                Err(Errno::EPERM)
            );
        });
        ctx.as_user(another_user, None, || {
            assert_eq!(
                lchown(&file, Some(user.uid), Some(user.gid)),
                Err(Errno::EPERM)
            );
        });
        ctx.as_user(another_user, None, || {
            assert_eq!(
                lchown(&file, Some(another_user.uid), Some(another_user.gid)),
                Err(Errno::EPERM)
            );
        });
        ctx.as_user(user, None, || {
            assert_eq!(
                lchown(&file, None, Some(another_user.gid)),
                Err(Errno::EPERM)
            );
        });
    }

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

    // chown/09.t
    erofs_named_test_case!(lchown, lchown_wrapper);

    // chown/10.t
    efault_path_test_case!(lchown, |ptr| nix::libc::lchown(ptr, 0, 0));
}
