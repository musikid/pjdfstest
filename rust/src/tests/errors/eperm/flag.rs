use std::{fs::metadata, os::unix::prelude::FileTypeExt, path::Path};

use nix::{
    sys::stat::{mknod, stat, Mode, SFlag},
    unistd::{mkdir, mkfifo, truncate, unlink},
};

use crate::utils::{chmod, link, symlink, ALLPERMS};

/// Asserts that the syscall returns EPERM if the file has one of flag defined by the user to fail with this error
/// in the configuration file, with [`EpermConfig`](crate::config::EpermConfig).
/// The first parameter should be the name of the syscall, and the next ones should be a closure which takes the file path and return
/// the syscall's result and finally another closure which checks that the syscall effectively didn't change the file.
macro_rules! assert_eperm_flag_file {
    ($syscall: ident => |$path_fn: ident| $fn: block if |$path_check: ident| $check: block) => {
        paste::paste! {
            $crate::test_case! {
                #[doc = concat!(stringify!($syscall), " returns EPERM if the named file has one of the flag defined by the user set")]
                [<$syscall _file>], $crate::test::FileSystemFeature::Chflags
            }
            fn [<$syscall _file>](ctx: &mut $crate::runner::context::TestContext) {
                let syscall = stringify!($syscall);
                let f = |$path_fn: &std::path::Path| $fn;
                let check = |$path_check: &std::path::Path| $check;
                let flags: Vec<_> = ctx
                    .config()
                    .eperm
                    .syscalls_flags
                    .get(syscall)
                    .and_then(|flags| flags.file.as_ref().map(|fs| fs.iter().cloned().collect()))
                    .unwrap_or_default();

                if !flags.is_empty() {
                    print!("Testing for {}\t", flags.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "));
                } else {
                    print!("No flags set\t");
                }

                for flag in flags {
                    let file = ctx
                        .create(crate::runner::context::FileType::Regular)
                        .unwrap();
                    nix::unistd::chflags(&file, flag.into()).unwrap();
                    assert_eq!(f(&file), Err(nix::errno::Errno::EPERM), "{flag} does not trigger EPERM");
                    assert!(!check(&file), "Error file check failed");
                    nix::unistd::chflags(&file, nix::sys::stat::FileFlag::empty()).unwrap();
                    assert!(f(&file).is_ok());
                    assert!(check(&file), "Success file check failed");
                }
            }
        }
    };
}

/// Asserts that the syscall returns EPERM if the parent directory of the file has one of flag defined by the user to fail with this error
/// in the configuration file, with [`EpermConfig`](crate::config::EpermConfig).
/// The first parameter should be the name of the syscall, and the next ones should be a closure which takes the file path and return
/// the syscall's result and finally another closure which checks that the syscall effectively didn't change the file.
macro_rules! assert_eperm_flag_dir {
    ($syscall: ident, only_path => |$path_fn: ident| $fn: block if |$path_check: ident| $check: block) => {
        paste::paste! {
            $crate::test_case! {
                #[doc = concat!(stringify!($syscall), " returns EPERM if the named file has one of the flag defined by the user set")]
                [<$syscall _dir>], $crate::test::FileSystemFeature::Chflags
            }
            fn [<$syscall _dir>](ctx: &mut $crate::runner::context::TestContext) {
                let syscall = stringify!($syscall);
                let f = |$path_fn: &std::path::Path| $fn;
                let check = |$path_check: &std::path::Path| $check;
                let flags: Vec<_> = ctx
                    .config()
                    .eperm
                    .syscalls_flags
                    .get(syscall)
                    .and_then(|flags| flags.parent.as_ref().map(|fs| fs.iter().cloned().collect()))
                    .unwrap_or_default();

                if !flags.is_empty() {
                    print!("Testing for {}\t", flags.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "));
                } else {
                    print!("No flags set\t");
                }

                for flag in flags {
                    let dir = ctx.create(crate::runner::context::FileType::Dir).unwrap();
                    let file = dir.join("file");
                    nix::unistd::chflags(&dir, flag.into()).unwrap();
                    assert_eq!(f(&file), Err(nix::errno::Errno::EPERM), "{flag} does not trigger EPERM");
                    assert!(!check(&file), "Error file check failed");
                    nix::unistd::chflags(&dir, nix::sys::stat::FileFlag::empty()).unwrap();
                    assert!(f(&file).is_ok());
                    assert!(check(&file), "Success file check failed");
                }
            }
        }
    };
    ($syscall: ident => |$path_fn: ident| $fn: block if |$path_check: ident| $check: block) => {
        paste::paste! {
            $crate::test_case! {
                #[doc = concat!(stringify!($syscall), " returns EPERM if the named file has one of the flag defined by the user set")]
                [<$syscall _dir>], $crate::test::FileSystemFeature::Chflags
            }
            fn [<$syscall _dir>](ctx: &mut $crate::runner::context::TestContext) {
                let syscall = stringify!($syscall);
                let f = |$path_fn: &std::path::Path| $fn;
                let check = |$path_check: &std::path::Path| $check;
                let flags: Vec<_> = ctx
                    .config()
                    .eperm
                    .syscalls_flags
                    .get(syscall)
                    .and_then(|flags| flags.parent.as_ref().map(|fs| fs.iter().cloned().collect()))
                    .unwrap_or_default();

                if !flags.is_empty() {
                    print!("Testing for {}\t", flags.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "));
                } else {
                    print!("No flags set\t");
                }

                for flag in flags {
                    let dir = ctx.create(crate::runner::context::FileType::Dir).unwrap();
                    let file = ctx
                        .new_file(crate::runner::context::FileType::Regular)
                        .name(dir.join("file"))
                        .create()
                        .unwrap();
                    nix::unistd::chflags(&dir, flag.into()).unwrap();
                    assert_eq!(f(&file), Err(nix::errno::Errno::EPERM), "{flag} does not trigger EPERM for {syscall}");
                    assert!(!check(&file), "Error file check failed");
                    nix::unistd::chflags(&dir, nix::sys::stat::FileFlag::empty()).unwrap();
                    assert!(f(&file).is_ok());
                    assert!(check(&file), "Success file check failed");
                }
            }
        }
    };
}

// chmod/08.t
//TODO: lchmod
assert_eperm_flag_file! {chmod => |path| { chmod(path, Mode::from_bits_truncate(0o400)) } if |path| {
    let meta = stat(path).unwrap();
    meta.st_mode & ALLPERMS == 0o400
}}

// (f)truncate/08.t
assert_eperm_flag_file! {ftruncate => |path| { truncate(path, 123) } if |path| { let meta = stat(path).unwrap(); meta.st_size == 123 }}

// link/12.t
assert_eperm_flag_file! {link => |path| { let new_path = path.parent().unwrap().join("new_file"); link(path, &new_path) }
    if |path| { let nlinks = stat(path).unwrap().st_nlink; nlinks == 2 }
}

// link/13.t
assert_eperm_flag_dir! {link => |path| { let new_path = path.parent().unwrap().join("new_file"); link(path, &new_path) }
    if |path| { let nlinks = stat(path).unwrap().st_nlink; nlinks == 2 }
}

// mkdir/08.t
assert_eperm_flag_dir! {mkdir, only_path => |path| { mkdir(path, Mode::from_bits_truncate(0o755)) } if |path| {
    path.is_dir()
}}

// mkfifo/10.t
assert_eperm_flag_dir! {mkfifo, only_path => |path| { mkfifo(path, Mode::from_bits_truncate(0o755)) } if |path| {
    path.exists() && metadata(path).unwrap().file_type().is_fifo()
}}

// mknod/09.t
assert_eperm_flag_dir! {mknod, only_path => |path| { mknod(path, SFlag::S_IFIFO, Mode::from_bits_truncate(0o755), 0) } if |path| {
    path.exists() && metadata(path).unwrap().file_type().is_fifo()
}}

// symlink/09.t
assert_eperm_flag_dir! {symlink, only_path => |path| { symlink(Path::new("test"), path) }
    if |path| { path.is_symlink() }
}

// unlink/09.t
assert_eperm_flag_file! {unlink => |path| { unlink(path) } if |path| { !path.exists() } }

// unlink/10.t
assert_eperm_flag_dir! {unlink => |path| { unlink(path) } if |path| { !path.exists() } }
