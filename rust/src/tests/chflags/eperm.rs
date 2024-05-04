use nix::unistd::chflags;

pub(super) mod macros;

macros::immutable_append_nounlink_not_root_test_case!(chflags);
macros::set_immutable_append_nounlink_not_root_test_case!(chflags);
macros::not_owner_not_root_test_case!(chflags);
macros::set_sf_snapshot_user_test_case!(chflags);
