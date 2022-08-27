use nix::{
    fcntl::{open, OFlag},
    sys::stat::Mode,
};

// open/22.t
crate::eexist_test_case! {open, |_ctx, path|
    open(path, OFlag::O_CREAT | OFlag::O_EXCL, Mode::from_bits_truncate(0o644))
}
