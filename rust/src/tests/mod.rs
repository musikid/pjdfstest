use std::os::unix::fs::MetadataExt as StdMetadataExt;
use std::{
    fs::metadata,
    path::Path,
    time::{Duration, SystemTime}
};

use crate::test::TestContext;

pub mod chmod;

/// A handy extention to std::os::unix::fs::MetadataExt
trait MetadataExt: StdMetadataExt {
    /// Return the file's last changed time as a `SystemTime`, including
    /// fractional component.
    fn ctime_sys(&self) -> SystemTime {
        let nsec = u32::try_from(self.ctime_nsec())
            .expect("File has denormalized timestamp");
        if self.ctime() >= 0 {
            SystemTime::UNIX_EPOCH + Duration::new(self.ctime() as u64, nsec)
        } else {
            SystemTime::UNIX_EPOCH - Duration::from_secs(-self.ctime() as u64)
                + Duration::new(0, nsec)
        }
    }
}

impl<T: StdMetadataExt> MetadataExt for T {}

/// Assert that a certain operation changes the ctime of a file.
fn assert_ctime_changed<F>(ctx: &mut TestContext, path: &Path, f: F)
    where F: FnOnce()
{
    let ctime_before = metadata(&path).unwrap().ctime_sys();

    ctx.nap();

    f();

    let ctime_after = metadata(&path).unwrap().ctime_sys();
    assert!(ctime_after > ctime_before);
}

/// Assert that a certain operation does not change the ctime of a file.
fn assert_ctime_unchanged<F>(ctx: &TestContext, path: &Path, f: F)
    where F: FnOnce()
{
    let ctime_before = metadata(&path).unwrap().ctime_sys();

    ctx.nap();

    f();

    let ctime_after = metadata(&path).unwrap().ctime_sys();
    assert!(ctime_after == ctime_before);
}
