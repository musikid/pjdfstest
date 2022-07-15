use std::{
    fs::{metadata, symlink_metadata},
    os::unix::fs::symlink,
};

#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
use crate::tests::birthtime_ts;
use crate::tests::{chmod, MetadataExt};
use crate::{runner::context::FileType, test::TestContext};
use crate::{runner::context::SerializedTestContext, test::FileSystemFeature};

use nix::{
    errno::Errno,
    sys::{
        stat::{utimensat, Mode, UtimensatFlags::*},
        time::{TimeSpec, TimeValLike},
    },
    unistd::Uid,
};

const UTIME_NOW: TimeSpec = TimeSpec::new(0, libc::UTIME_NOW);
const UTIME_OMIT: TimeSpec = TimeSpec::new(0, libc::UTIME_OMIT);

crate::test_case! {
    /// utimensat changes timestamps on any type of file
    // utimensat/00.t
    changes_timestamps, FileSystemFeature::Utimensat => [Regular, Dir, Fifo, Block, Char, Socket]
}
fn changes_timestamps(ctx: &mut TestContext, f_type: FileType) {
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(f_type).unwrap();

    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();

    let md = metadata(&path).unwrap();
    assert_eq!(md.atime_ts(), date1);
    assert_eq!(md.mtime_ts(), date2);
}

crate::test_case! {
    /// utimensat with UTIME_NOW will set the timestamps to now
    // utimensat/01.t
    utime_now, FileSystemFeature::Utimensat, FileSystemFeature::UtimeNow
}
fn utime_now(ctx: &mut TestContext) {
    // Allow up to a 5 minute delta between timestamps
    let margin = TimeSpec::seconds(300);
    let null_ts = TimeSpec::seconds(0);
    let path = ctx.create(FileType::Regular).unwrap();
    let md = metadata(&path).unwrap();
    let orig_atime = md.atime_ts();
    let orig_mtime = md.mtime_ts();
    ctx.nap();

    utimensat(None, &path, &UTIME_NOW, &UTIME_NOW, FollowSymlink).unwrap();

    let md = metadata(&path).unwrap();
    let delta_atime = md.atime_ts() - orig_atime;
    let delta_mtime = md.mtime_ts() - orig_mtime;
    assert!(delta_atime > null_ts, "atime was not updated");
    assert!(delta_mtime > null_ts, "mtime was not updated");
    assert!(
        delta_atime < margin,
        "new atime is implausibly far in the future"
    );
    assert!(
        delta_mtime < margin,
        "new mtime is implausibly far in the future"
    );
}

crate::test_case! {
    /// utimensat with UTIME_OMIT will leave the timestamps unchanged
    // utimensat/02.t
    utime_omit, FileSystemFeature::Utimensat
}
fn utime_omit(ctx: &mut TestContext) {
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(FileType::Regular).unwrap();
    let md = metadata(&path).unwrap();
    let orig_mtime = md.mtime_ts();

    utimensat(None, &path, &date1, &UTIME_OMIT, FollowSymlink).unwrap();
    let md = metadata(&path).unwrap();
    assert_eq!(md.atime_ts(), date1);
    assert_eq!(md.mtime_ts(), orig_mtime);

    utimensat(None, &path, &UTIME_OMIT, &date2, FollowSymlink).unwrap();
    let md = metadata(&path).unwrap();
    assert_eq!(md.atime_ts(), date1);
    assert_eq!(md.mtime_ts(), date2);
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
crate::test_case! {
    /// utimensat can update birthtimes
    // utimensat/03.t
    birthtime, FileSystemFeature::Utimensat, FileSystemFeature::StatStBirthtime
}
#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn birthtime(ctx: &mut TestContext) {
    let date1 = TimeSpec::seconds(100000000); // Sat Mar  3 02:46:40 MST 1973
    let date2 = TimeSpec::seconds(200000000); // Mon May  3 13:33:20 MDT 1976
    let path = ctx.create(FileType::Regular).unwrap();

    utimensat(None, &path, &date1, &date1, FollowSymlink).unwrap();
    let md = metadata(&path).unwrap();
    assert_eq!(date1, md.atime_ts());
    assert_eq!(date1, md.mtime_ts());
    assert_eq!(date1, birthtime_ts(&path));

    utimensat(None, &path, &date2, &date2, FollowSymlink).unwrap();
    let md = metadata(&path).unwrap();
    assert_eq!(date2, md.atime_ts());
    assert_eq!(date2, md.mtime_ts());
    assert_eq!(date1, birthtime_ts(&path));
}

crate::test_case! {
    /// utimensat can set mtime < atime or vice versa
    // utimensat/04.t
    order, FileSystemFeature::Utimensat
}
fn order(ctx: &mut TestContext) {
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(FileType::Regular).unwrap();
    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();
    utimensat(None, &path, &date2, &date1, FollowSymlink).unwrap();
}

crate::test_case! {
    /// utimensat can follow symlinks
    // utimensat/05.t
    follow_symlink, FileSystemFeature::Utimensat
}
fn follow_symlink(ctx: &mut TestContext) {
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let date3 = TimeSpec::seconds(1960000000); // Mon Feb  9 21:26:40 MST 2032
    let date4 = TimeSpec::seconds(1970000000); // Fri Jun  4 16:13:20 MDT 2032
    let date5 = TimeSpec::seconds(1980000000); // Tue Sep 28 10:00:00 MDT 2032
    let date6 = TimeSpec::seconds(1990000000); // Sat Jan 22 02:46:40 MST 2033

    let path = ctx.create(FileType::Regular).unwrap();
    let lpath = path.with_extension("link");
    symlink(&path, &lpath).unwrap();

    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();
    utimensat(None, &lpath, &date3, &date4, NoFollowSymlink).unwrap();

    let md = metadata(&path).unwrap();
    let lmd = symlink_metadata(&lpath).unwrap();
    assert_eq!(date1, md.atime_ts());
    assert_eq!(date2, md.mtime_ts());
    assert_eq!(date3, lmd.atime_ts());
    assert_eq!(date4, lmd.mtime_ts());

    utimensat(None, &lpath, &date5, &date6, FollowSymlink).unwrap();
    let md = metadata(&path).unwrap();
    let lmd = symlink_metadata(&lpath).unwrap();
    assert_eq!(date5, md.atime_ts());
    assert_eq!(date6, md.mtime_ts());
    // If atime is disabled on the current mount, then lpath's atime should
    // still be date3.  However, if atime is enabled, then lpath's atime will
    // be the current system time.  For this test, it's sufficient to simply
    // check that it didn't get set to date5.
    assert!(date5 != lmd.atime_ts());
    assert_eq!(date4, lmd.mtime_ts());
}

crate::test_case! {
    /// A user without write permission cannot use UTIME_NOW
    // utimensat/06.t:L26
    utime_now_nobody, serialized, FileSystemFeature::Utimensat, FileSystemFeature::UtimeNow
}
fn utime_now_nobody(ctx: &mut SerializedTestContext) {
    let mode = Mode::from_bits_truncate(0o644);
    let path = ctx.create(FileType::Regular).unwrap();
    chmod(&path, mode).unwrap();
    ctx.as_user(Some(Uid::from_raw(65534)), None, || {
        assert_eq!(
            Errno::EACCES,
            utimensat(None, &path, &UTIME_NOW, &UTIME_NOW, FollowSymlink).unwrap_err()
        );
    });
}

crate::test_case! {
    /// The file's owner can use UTIME_NOW, even if the file is read-only
    // utimensat/06.t:L30
    utime_now_owner, FileSystemFeature::Utimensat, FileSystemFeature::UtimeNow
}
fn utime_now_owner(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Regular).unwrap();
    let mode = Mode::from_bits_truncate(0o444);
    chmod(&path, mode).unwrap();
    utimensat(None, &path, &UTIME_NOW, &UTIME_NOW, FollowSymlink).unwrap();
}

crate::test_case! {
    /// The superuser can always use UTIME_NOW
    // utimensat/06.t:L35
    utime_now_root, root, FileSystemFeature::Utimensat, FileSystemFeature::UtimeNow
}
fn utime_now_root(ctx: &mut TestContext) {
    let path = ctx.create(FileType::Regular).unwrap();
    let mode = Mode::from_bits_truncate(0o444);
    chmod(&path, mode).unwrap();
    utimensat(None, &path, &UTIME_NOW, &UTIME_NOW, FollowSymlink).unwrap();
}

crate::test_case! {
    /// A user with write permission can use UTIME_NOW
    // utimensat/06.t:L38
    utime_now_write_perm, serialized, FileSystemFeature::Utimensat, FileSystemFeature::UtimeNow
}
fn utime_now_write_perm(ctx: &mut SerializedTestContext) {
    let mode = Mode::from_bits_truncate(0o666);
    let path = ctx.create(FileType::Regular).unwrap();
    chmod(&path, mode).unwrap();
    ctx.as_user(Some(Uid::from_raw(65534)), None, || {
        utimensat(None, &path, &UTIME_OMIT, &UTIME_OMIT, FollowSymlink).unwrap();
    });
}

crate::test_case! {
    /// A user without write permission cannot set the timestamps arbitrarily
    // utimensat/07.t:L28
    nobody, serialized, FileSystemFeature::Utimensat
}
fn nobody(ctx: &mut SerializedTestContext) {
    let mode = Mode::from_bits_truncate(0o644);
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(FileType::Regular).unwrap();
    chmod(&path, mode).unwrap();
    ctx.as_user(Some(Uid::from_raw(65534)), None, || {
        assert_eq!(
            Errno::EPERM,
            utimensat(None, &path, &UTIME_OMIT, &date2, FollowSymlink).unwrap_err()
        );
        assert_eq!(
            Errno::EPERM,
            utimensat(None, &path, &date1, &UTIME_OMIT, FollowSymlink).unwrap_err()
        );
        assert_eq!(
            Errno::EPERM,
            utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap_err()
        );
    })
}

crate::test_case! {
    /// A user with write permission cannot set the timestamps arbitrarily
    // utimensat/07.t:L33
    write_perm, serialized, FileSystemFeature::Utimensat
}
fn write_perm(ctx: &mut SerializedTestContext) {
    let mode = Mode::from_bits_truncate(0o666);
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(FileType::Regular).unwrap();
    chmod(&path, mode).unwrap();
    ctx.as_user(Some(Uid::from_raw(65534)), None, || {
        assert_eq!(
            Errno::EPERM,
            utimensat(None, &path, &UTIME_OMIT, &date2, FollowSymlink).unwrap_err()
        );
        assert_eq!(
            Errno::EPERM,
            utimensat(None, &path, &date1, &UTIME_OMIT, FollowSymlink).unwrap_err()
        );
        assert_eq!(
            Errno::EPERM,
            utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap_err()
        );
    })
}

crate::test_case! {
    /// The owner can update the timstamps, even if the file is read-only
    // utimensat/07.t:L40
    owner, FileSystemFeature::Utimensat
}
fn owner(ctx: &mut TestContext) {
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(FileType::Regular).unwrap();
    let mode = Mode::from_bits_truncate(0o444);
    chmod(&path, mode).unwrap();
    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();
}
crate::test_case! {
    /// Root can always update the timestamps, even if the file is read-only
    // utimensat/07.t:L44
    root, root, FileSystemFeature::Utimensat
}
fn root(ctx: &mut TestContext) {
    let date1 = TimeSpec::seconds(1900000000); // Sun Mar 17 11:46:40 MDT 2030
    let date2 = TimeSpec::seconds(1950000000); // Fri Oct 17 04:40:00 MDT 2031
    let path = ctx.create(FileType::Regular).unwrap();
    let mode = Mode::from_bits_truncate(0o444);
    chmod(&path, mode).unwrap();
    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();
}

crate::test_case! {
    /// utimensat can set timestamps with subsecond precision
    // utimensat/08.t
    subsecond, FileSystemFeature::Utimensat
}
fn subsecond(ctx: &mut TestContext) {
    // Different file systems have different timestamp resolutions.  Check that
    // they can do 0.1 second, but don't bother checking the finest resolution.

    // Sat Mar  3 02:46:40 MST 1973
    let date1 = TimeSpec::new(100000000, 100000000);
    // Mon May  3 13:33:20 MDT 1976
    let date2 = TimeSpec::new(200000000, 200000000);

    let path = ctx.create(FileType::Regular).unwrap();

    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();

    let md = metadata(&path).unwrap();
    assert_eq!(date1, md.atime_ts());
    assert_eq!(date2, md.mtime_ts());
}

crate::test_case! {
    /// utimensat is y2038 compliant
    // utimensat/09.t
    y2038, FileSystemFeature::Utimensat
}
fn y2038(ctx: &mut TestContext) {
    // 2^31, ie Mon Jan 18 20:14:08 MST 2038
    let date1 = TimeSpec::seconds(2147483648);
    // 2^32, ie Sat Feb  6 23:28:16 MST 2106
    let date2 = TimeSpec::seconds(4294967296);

    let path = ctx.create(FileType::Regular).unwrap();

    utimensat(None, &path, &date1, &date2, FollowSymlink).unwrap();

    let md = metadata(&path).unwrap();
    assert_eq!(date1, md.atime_ts());
    assert_eq!(date2, md.mtime_ts());
}
