use std::{fs::metadata, os::freebsd::fs::MetadataExt as _};

use nix::{
    errno::Errno,
    fcntl::{fcntl, open, FcntlArg, OFlag},
    sys::stat::{FileFlag, Mode},
    unistd::chflags,
};

use crate::{
    context::{FileType, TestContext},
    features::FileSystemFeature,
    flags::FileFlags,
    tests::errors::eperm::flag::{get_supported_and_error_flags, supports_any_flag},
};

crate::test_case! {
    /// open returns EPERM when the named file has its immutable flag set
    /// and the file is to be modified
    // open/10.t
    immutable_file, root, FileSystemFeature::Chflags; supports_any_flag!(FileFlags::IMMUTABLE_FLAGS)
}
fn immutable_file(ctx: &mut TestContext) {
    // In the original test the flags considered as valid are the undeletable ones,
    // hence why we don't take the valid flags from `get_flags_intersection`
    let (flags, _) = get_supported_and_error_flags(
        &ctx.features_config().file_flags,
        FileFlags::IMMUTABLE_FLAGS,
    );
    let valid_flags = FileFlags::UNDELETABLE_FLAGS.iter().copied().collect();
    let valid_flags: Vec<_> = ctx
        .features_config()
        .file_flags
        .intersection(&valid_flags)
        .copied()
        .collect();

    let oflags = [
        OFlag::O_WRONLY,
        OFlag::O_RDWR,
        OFlag::O_RDONLY | OFlag::O_TRUNC,
    ];

    for flag in flags {
        let raw_flag: FileFlag = flag.into();
        let file = ctx.create(FileType::Regular).unwrap();

        chflags(&file, raw_flag).unwrap();

        let set_flags = metadata(&file).unwrap().st_flags();
        assert!(
            set_flags as u64 & raw_flag.bits() > 0,
            "File should have {flag} set but only have {set_flags}"
        );

        for oflag in oflags {
            assert!(
                matches!(open(&file, oflag, Mode::empty()), Err(Errno::EPERM)),
                "Opening with {oflag:?} for {flag} does not trigger EPERM"
            );
        }
    }

    for flag in valid_flags {
        let raw_flag: FileFlag = flag.into();
        let file = ctx.create(FileType::Regular).unwrap();

        chflags(&file, raw_flag).unwrap();

        for oflag in oflags {
            assert!(
                open(&file, oflag, Mode::empty()).is_ok(),
                "Failure when checking if open with {oflag:?} is working for valid flag {flag}"
            );
        }
    }
}

crate::test_case! {
    /// open returns EPERM when the named file has its append-only flag set,
    /// the file is to be modified, and O_TRUNC is specified or O_APPEND is not specified
    // open/11.t
    append_file, FileSystemFeature::Chflags; supports_any_flag!(FileFlags::APPEND_ONLY_FLAGS)
}
fn append_file(ctx: &mut TestContext) {
    let (flags, _) = get_supported_and_error_flags(
        &ctx.features_config().file_flags,
        FileFlags::APPEND_ONLY_FLAGS,
    );

    let invalid_oflags = [
        OFlag::O_WRONLY,
        OFlag::O_RDWR,
        OFlag::O_RDONLY | OFlag::O_TRUNC,
        OFlag::O_RDONLY | OFlag::O_APPEND | OFlag::O_TRUNC,
        OFlag::O_WRONLY | OFlag::O_APPEND | OFlag::O_TRUNC,
        OFlag::O_RDWR | OFlag::O_APPEND | OFlag::O_TRUNC,
    ];
    let valid_oflags = [
        OFlag::O_WRONLY | OFlag::O_APPEND,
        OFlag::O_RDWR | OFlag::O_APPEND,
    ];

    for flag in flags {
        let file = ctx.create(FileType::Regular).unwrap();
        let raw_flag: FileFlag = flag.into();

        chflags(&file, raw_flag).unwrap();

        let set_flags = metadata(&file).unwrap().st_flags();
        assert!(
            set_flags as u64 & raw_flag.bits() > 0,
            "File should have {flag} set but only have {set_flags}"
        );

        for oflag in invalid_oflags {
            let res = open(&file, oflag, Mode::empty());
            assert!(
                matches!(res, Err(Errno::EPERM)),
                "Opening file with {oflag:?} for flag {flag} does not trigger EPERM"
            );
        }

        for oflag in valid_oflags {
            let res = open(&file, oflag, Mode::empty());
            assert!(
                res.is_ok(),
                "Opening file with {oflag:?} for flag {flag} does not work"
            );
            let fd = res.unwrap();
            assert!(
                fcntl(fd, FcntlArg::F_GETFD).is_ok(),
                "Opened file descriptor check failed for {flag}"
            );
        }
    }
}
