use std::{path::Path, str::FromStr};

use exacl::{AclEntry, AclOption};

mod append;
mod chown;
mod delete;
mod delete_child;
mod readattr;
mod readsecurity;
mod write_data;
mod writesecurity;

/// Prepend an ACL entry to a file's existing access control list.
fn prependacl<P: AsRef<Path>>(path: P, spec: &str) {
    let entry = AclEntry::from_str(spec).unwrap();
    let mut entries = exacl::getfacl(&path, AclOption::empty()).unwrap();
    let mut new_entries = vec![entry];
    new_entries.append(&mut entries);
    exacl::setfacl(&[path][..], &new_entries, AclOption::empty()).unwrap();
}
