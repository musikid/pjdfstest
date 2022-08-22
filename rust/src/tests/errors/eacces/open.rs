use std::{fs::File, io::Write, path::Path};

use nix::{
    errno::Errno,
    fcntl::{open, OFlag},
    sys::stat::Mode,
    unistd::{chown, User},
};

use crate::{
    runner::context::{FileType, SerializedTestContext},
    utils::chmod,
};

#[derive(Debug)]
enum AssertAs {
    User,
    Group,
    Other,
}

fn assert_mode_flag(
    ctx: &SerializedTestContext,
    file: &Path,
    first_user: &User,
    other_user: &User,
    mode: Mode,
    assert_as: AssertAs,
    flags_ok: &[OFlag],
    flags_err: &[OFlag],
) {
    let user = match assert_as {
        AssertAs::User => first_user,
        _ => other_user,
    };
    let gid = match assert_as {
        AssertAs::Group => first_user.gid,
        _ => user.gid,
    };

    chmod(file, mode).unwrap();
    ctx.as_user(user, Some(std::slice::from_ref(&gid)), || {
        for flag_ok in flags_ok {
            assert!(
                open(file, *flag_ok, Mode::empty()).is_ok(),
                "open with flags {flag_ok:?} for mode {mode:?} as {assert_as:?} 
                has failed when it should succeed"
            );
        }
        for flag_err in flags_err {
            assert_eq!(
                open(file, *flag_err, Mode::empty()),
                Err(Errno::EACCES),
                "open with flags {flag_err:?} for mode {mode:?} as {assert_as:?} 
                has succeeded when it should fail"
            );
        }
    });
}

crate::test_case! {
    /// open returns EACCES when the required permissions (for reading and/or writing) are denied for the given flags
    // open/06.t
    required_perms_denied_flags, serialized, root => [Regular, Fifo, Dir]
}
fn required_perms_denied_flags(ctx: &mut SerializedTestContext, ft: FileType) {
    let file = ctx.create(ft.clone()).unwrap();
    let first_user = ctx.get_new_user();
    let other_user = ctx.get_new_user();
    chown(&file, Some(first_user.uid), Some(first_user.gid)).unwrap();

    let assert_mode = |mode, assert_as, flags_ok, flags_err| {
        assert_mode_flag(
            ctx,
            &file,
            &first_user,
            &other_user,
            mode,
            assert_as,
            flags_ok,
            flags_err,
        )
    };

    let assert_mode_flag_ok =
        |mode, assert_as, flags_ok| assert_mode(mode, assert_as, flags_ok, &[]);
    let assert_mode_flag_err =
        |mode, assert_as, flags_err| assert_mode(mode, assert_as, &[], flags_err);

    // Read/write permissions

    // BUG: Using a bit or and inlining the value trigger a lifetime error
    const RDONLY_NONBLOCK: OFlag = OFlag::O_RDONLY.union(OFlag::O_NONBLOCK);

    let flags: &[OFlag] = match ft {
        FileType::Fifo => &[RDONLY_NONBLOCK, OFlag::O_RDWR],
        FileType::Dir => &[OFlag::O_RDONLY],
        FileType::Regular => &[OFlag::O_RDONLY, OFlag::O_WRONLY, OFlag::O_RDWR],
        _ => unreachable!(),
    };

    // User
    assert_mode_flag_ok(Mode::S_IRUSR | Mode::S_IWUSR, AssertAs::User, flags);
    // Group
    assert_mode_flag_ok(Mode::S_IRGRP | Mode::S_IWGRP, AssertAs::Group, flags);
    // Other
    assert_mode_flag_ok(Mode::S_IROTH | Mode::S_IWOTH, AssertAs::Other, flags);

    // Read-only

    // BUG: Using a bit or and inlining the value trigger a lifetime error
    const WRONLY_NONBLOCK: OFlag = OFlag::O_WRONLY.union(OFlag::O_NONBLOCK);

    let flags_err: &[OFlag] = match ft {
        FileType::Fifo => &[WRONLY_NONBLOCK, OFlag::O_RDWR],
        FileType::Dir => &[],
        FileType::Regular => &[OFlag::O_WRONLY, OFlag::O_RDWR],
        _ => unreachable!(),
    };

    let flags_ok: &[OFlag] = match ft {
        FileType::Fifo => &[RDONLY_NONBLOCK],
        FileType::Dir => &[OFlag::O_RDONLY],
        FileType::Regular => &[OFlag::O_RDONLY],
        _ => unreachable!(),
    };

    // User
    assert_mode(
        Mode::S_IRUSR | Mode::S_IRWXG | Mode::S_IRWXO,
        AssertAs::User,
        flags_ok,
        flags_err,
    );
    // Group
    assert_mode(
        Mode::S_IRWXU | Mode::S_IRGRP | Mode::S_IRWXO,
        AssertAs::Group,
        flags_ok,
        flags_err,
    );
    // Other
    assert_mode(
        Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IROTH,
        AssertAs::Other,
        flags_ok,
        flags_err,
    );

    // Write-only

    let flags_err: &[OFlag] = match ft {
        FileType::Fifo => &[RDONLY_NONBLOCK, OFlag::O_RDWR],
        FileType::Dir => &[],
        FileType::Regular => &[OFlag::O_RDONLY, OFlag::O_RDWR],
        _ => unreachable!(),
    };

    let flags_ok: &[OFlag] = match ft {
        FileType::Fifo => &[WRONLY_NONBLOCK],
        FileType::Dir => &[],
        FileType::Regular => &[OFlag::O_WRONLY],
        _ => unreachable!(),
    };

    // User
    assert_mode(
        Mode::S_IWUSR | Mode::S_IRWXG | Mode::S_IRWXO,
        AssertAs::User,
        flags_ok,
        flags_err,
    );
    // Group
    assert_mode(
        Mode::S_IRWXU | Mode::S_IWGRP | Mode::S_IRWXO,
        AssertAs::Group,
        flags_ok,
        flags_err,
    );
    // Other
    assert_mode(
        Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IWOTH,
        AssertAs::Other,
        flags_ok,
        flags_err,
    );

    // Execute-only

    let flags: &[OFlag] = match ft {
        FileType::Fifo => &[RDONLY_NONBLOCK, WRONLY_NONBLOCK, OFlag::O_RDWR],
        FileType::Dir => &[OFlag::O_RDONLY],
        FileType::Regular => &[OFlag::O_RDONLY, OFlag::O_WRONLY, OFlag::O_RDWR],
        _ => unreachable!(),
    };

    // User
    assert_mode_flag_err(
        Mode::S_IXUSR | Mode::S_IRWXG | Mode::S_IRWXO,
        AssertAs::User,
        flags,
    );
    // Group
    assert_mode_flag_err(
        Mode::S_IRWXU | Mode::S_IXGRP | Mode::S_IRWXO,
        AssertAs::Group,
        flags,
    );
    // Other
    assert_mode_flag_err(
        Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IXOTH,
        AssertAs::Other,
        flags,
    );

    // No rights

    // User
    assert_mode_flag_err(Mode::S_IRWXG | Mode::S_IRWXO, AssertAs::User, flags);
    // Group
    assert_mode_flag_err(Mode::S_IRWXU | Mode::S_IRWXO, AssertAs::Group, flags);
    // Other
    assert_mode_flag_err(Mode::S_IRWXU | Mode::S_IRWXG, AssertAs::Other, flags);
}

crate::test_case! {
    /// open returns EACCES when O_TRUNC is specified and write permission is denied
    // open/07.t
    write_perm_o_trunc, serialized, root
}
fn write_perm_o_trunc(ctx: &mut SerializedTestContext) {
    let path = ctx.create(FileType::Regular).unwrap();
    let first_user = ctx.get_new_user();
    let other_user = ctx.get_new_user();
    chown(&path, Some(first_user.uid), Some(first_user.gid)).unwrap();

    let mut file = File::create(&path).unwrap();
    const EXAMPLE_BYTES: &str = "Hello";
    file.write_all(EXAMPLE_BYTES.as_bytes()).unwrap();
    let meta = path.metadata().unwrap();
    assert_eq!(meta.len(), EXAMPLE_BYTES.len() as u64);

    let assert_mode_err = |mode, assert_as, flags_err| {
        assert_mode_flag(
            ctx,
            &path,
            &first_user,
            &other_user,
            mode,
            assert_as,
            &[],
            flags_err,
        )
    };

    let flags = &[OFlag::O_RDONLY | OFlag::O_TRUNC];

    // Write-only
    // User
    assert_mode_err(
        Mode::S_IWUSR | Mode::S_IRWXG | Mode::S_IRWXO,
        AssertAs::User,
        flags,
    );
    // Group
    assert_mode_err(
        Mode::S_IRWXU | Mode::S_IWGRP | Mode::S_IRWXO,
        AssertAs::Group,
        flags,
    );
    // Other
    assert_mode_err(
        Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IWOTH,
        AssertAs::Other,
        flags,
    );

    // Execute-only
    // User
    assert_mode_err(
        Mode::S_IXUSR | Mode::S_IRWXG | Mode::S_IRWXO,
        AssertAs::User,
        flags,
    );
    // Group
    assert_mode_err(
        Mode::S_IRWXU | Mode::S_IXGRP | Mode::S_IRWXO,
        AssertAs::Group,
        flags,
    );
    // Other
    assert_mode_err(
        Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IXOTH,
        AssertAs::Other,
        flags,
    );

    // No rights

    // User
    assert_mode_err(Mode::S_IRWXG | Mode::S_IRWXO, AssertAs::User, flags);
    // Group
    assert_mode_err(Mode::S_IRWXU | Mode::S_IRWXO, AssertAs::Group, flags);
    // Other
    assert_mode_err(Mode::S_IRWXU | Mode::S_IRWXG, AssertAs::Other, flags);
}

crate::test_case! {
    /// open returns EACCES when O_TRUNC is specified and write permission is denied
    // open/07.t
    o_creat_parent_write_perm, serialized, root
}
fn o_creat_parent_write_perm(ctx: &mut SerializedTestContext) {
    let file = ctx.create(FileType::Regular).unwrap();
    assert_eq!(
        open(
            &file,
            OFlag::O_RDONLY | OFlag::O_CREAT,
            Mode::S_IRUSR | Mode::S_IWUSR
        ),
        Err(Errno::EACCES)
    );
}
