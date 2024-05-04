mod eperm {
    use super::super::eperm::macros;
    use crate::utils::lchflags;

    macros::immutable_append_nounlink_not_root_test_case!(lchflags, symlink);
    macros::set_immutable_append_nounlink_not_root_test_case!(lchflags, symlink);
    macros::not_owner_not_root_test_case!(lchflags, symlink);
    macros::set_sf_snapshot_user_test_case!(lchflags, symlink);
}
