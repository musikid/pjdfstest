use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // OS-exclusive syscalls
        chflags: { any(target_os = "openbsd", target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly", target_os = "macos", target_os = "ios") },
        lchmod: { any(target_os = "netbsd", target_os = "freebsd", target_os = "dragonfly") },
        lchflags: { any(target_os = "openbsd", target_os = "netbsd", target_os = "freebsd",
                    target_os = "dragonfly", target_os = "macos", target_os = "ios") },
        // OS-exclusive features
        file_flags: { any(target_os = "openbsd", target_os = "netbsd", target_os = "freebsd",
                    target_os = "dragonfly", target_os = "macos", target_os = "ios") },
        birthtime: { any(target_os = "freebsd", target_os = "ios", target_os = "macos", target_os = "netbsd", target_os = "openbsd") }
    }
}
