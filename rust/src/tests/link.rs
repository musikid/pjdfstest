use nix::{
    errno::Errno,
    sys::stat::{lstat, Mode},
    unistd::{chown, unlink},
};

use crate::{
    runner::context::{FileType, SerializedTestContext, TestContext},
    tests::AsTimeInvariant,
    utils::{chmod, link},
};

use super::{
    assert_ctime_changed, assert_ctime_unchanged, assert_mtime_changed, assert_mtime_unchanged,
};

crate::test_case! {
    /// link creates hardlinks which share the same metadata
    // link/00.t#23-41
    share_metadata => [Regular, Fifo, Block, Char, Socket]
}
fn share_metadata(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let file_stat = lstat(&file).unwrap();
    assert_eq!(file_stat.st_nlink, 1);

    let first_link = ctx.gen_path();
    link(&file, &first_link).unwrap();
    let file_stat = lstat(&file).unwrap();
    let first_link_stat = lstat(&first_link).unwrap();
    assert_eq!(file_stat.st_nlink, 2);
    assert_eq!(file_stat.st_nlink, first_link_stat.st_nlink);
    assert_eq!(
        file_stat.as_time_invariant(),
        first_link_stat.as_time_invariant()
    );

    let second_link = ctx.gen_path();
    link(&first_link, &second_link).unwrap();

    let file_stat = lstat(&file).unwrap();
    let first_link_stat = lstat(&first_link).unwrap();
    let second_link_stat = lstat(&second_link).unwrap();
    assert_eq!(file_stat.st_nlink, 3);
    assert_eq!(file_stat.st_nlink, first_link_stat.st_nlink);
    assert_eq!(
        file_stat.as_time_invariant(),
        first_link_stat.as_time_invariant()
    );
    assert_eq!(
        first_link_stat.as_time_invariant(),
        second_link_stat.as_time_invariant()
    );

    chmod(&first_link, Mode::from_bits_truncate(0o201)).unwrap();
    let user = ctx.get_new_user();
    let group = ctx.get_new_group();
    chown(&first_link, Some(user.uid), Some(group.gid)).unwrap();

    let first_link_stat = lstat(&first_link).unwrap();
    let file_stat = lstat(&file).unwrap();
    let second_link_stat = lstat(&second_link).unwrap();
    assert_eq!(
        file_stat.as_time_invariant(),
        first_link_stat.as_time_invariant()
    );
    assert_eq!(
        first_link_stat.as_time_invariant(),
        second_link_stat.as_time_invariant()
    );
}

crate::test_case! {
    /// Removing a link should only change the number of links
    // link/00.t
    remove_link => [Regular, Fifo, Block, Char, Socket]
}
fn remove_link(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let first_link = ctx.gen_path();
    let second_link = ctx.gen_path();

    link(&file, &first_link).unwrap();
    link(&first_link, &second_link).unwrap();

    unlink(&file).unwrap();
    assert_eq!(lstat(&file).unwrap_err(), Errno::ENOENT);

    let first_link_stat = lstat(&first_link).unwrap();
    let second_link_stat = lstat(&second_link).unwrap();
    assert_eq!(first_link_stat.st_nlink, 2);
    assert_eq!(
        first_link_stat.as_time_invariant(),
        second_link_stat.as_time_invariant()
    );

    unlink(&second_link).unwrap();
    assert_eq!(lstat(&second_link).unwrap_err(), Errno::ENOENT);

    let first_link_stat = lstat(&first_link).unwrap();
    assert_eq!(first_link_stat.st_nlink, 1);

    unlink(&first_link).unwrap();
    assert_eq!(lstat(&first_link).unwrap_err(), Errno::ENOENT);
}

crate::test_case! {
    /// link changes ctime of file along with ctime and mtime of parent when sucessful
    // link/00.t
    changed_ctime_success => [Regular, Fifo, Block, Char, Socket]
}
fn changed_ctime_success(ctx: &mut TestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let new_path = ctx.gen_path();

    //TODO: Migrate to new time assertion api when merged
    assert_ctime_changed(ctx, &file, || {
        assert_ctime_changed(ctx, ctx.base_path(), || {
            assert_mtime_changed(ctx, ctx.base_path(), || {
                assert!(link(&file, &new_path).is_ok());
            });
        });
    })
}
crate::test_case! {
    /// link changes neither ctime of file nor ctime or mtime of parent when it fails
    // link/00.t#77
    unchanged_ctime_fails, serialized, root => [Regular, Fifo, Block, Char, Socket]
}
fn unchanged_ctime_fails(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft).unwrap();
    let new_path = ctx.gen_path();

    let user = ctx.get_new_user();
    //TODO: Migrate to new time assertion api when merged
    assert_ctime_unchanged(ctx, &file, || {
        assert_ctime_unchanged(ctx, ctx.base_path(), || {
            assert_mtime_unchanged(ctx, ctx.base_path(), || {
                ctx.as_user(&user, None, || {
                    assert!(matches!(
                        link(&file, &new_path),
                        Err(Errno::EPERM | Errno::EACCES)
                    ));
                })
            });
        });
    })
}
