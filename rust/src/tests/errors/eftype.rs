use nix::{
    errno::Errno,
    sys::stat::{lstat, stat, Mode},
    unistd::chown,
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    utils::{chmod, ALLPERMS_STICKY},
};

crate::test_case! {
    /// chmod returns EFTYPE if the effective user ID is not the super-user, the mode includes the sticky bit (S_ISVTX), and path does not refer to a directory
    // chmod/12.t
    eftype, serialized, root => [Regular, Fifo, Block, Char, Socket]
}
fn eftype(ctx: &mut SerializedTestContext, ft: FileType) {
    let user = ctx.get_new_user();

    let original_mode = Mode::from_bits_truncate(0o640);
    let file = ctx
        .new_file(ft)
        .mode(original_mode.bits())
        .create()
        .unwrap();
    chown(&file, Some(user.uid), Some(user.gid)).unwrap();
    let new_mode = Mode::from_bits_truncate(0o644);
    let link = ctx.create(FileType::Symlink(Some(file.clone()))).unwrap();

    // TODO: Should be configured by the user? What to do with other OS?
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    {
        use crate::utils::lchmod;

        ctx.as_user(&user, None, || {
            assert_eq!(chmod(&file, new_mode | Mode::S_ISVTX), Err(Errno::EFTYPE));
        });
        let file_stat = stat(&file).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());

        ctx.as_user(&user, None, || {
            assert_eq!(
                chmod(&link, original_mode | Mode::S_ISVTX),
                Err(Errno::EFTYPE)
            );
        });
        let file_stat = stat(&link).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());

        // lchmod

        let mode = Mode::from_bits_truncate(0o621) | Mode::S_ISVTX;
        ctx.as_user(&user, None, || {
            assert_eq!(lchmod(&file, mode), Err(Errno::EFTYPE));
        });

        let file_stat = lstat(&file).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        ctx.as_user(&user, None, || {
            assert!(chmod(&file, new_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&file).unwrap();
        assert_eq!(
            file_stat.st_mode & ALLPERMS_STICKY,
            (new_mode | Mode::S_ISVTX).bits()
        );

        ctx.as_user(&user, None, || {
            assert!(chmod(&link, original_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&link).unwrap();
        assert_eq!(
            file_stat.st_mode & ALLPERMS_STICKY,
            (original_mode | Mode::S_ISVTX).bits()
        );
    }

    #[cfg(any(target_os = "solaris"))]
    {
        ctx.as_user(&user, None, || {
            assert!(chmod(&file, new_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&file).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, new_mode.bits());

        ctx.as_user(&user, None, || {
            assert!(chmod(&link, original_mode | Mode::S_ISVTX).is_ok());
        });
        let file_stat = stat(&link).unwrap();
        assert_eq!(file_stat.st_mode & ALLPERMS_STICKY, original_mode.bits());
    }
}
