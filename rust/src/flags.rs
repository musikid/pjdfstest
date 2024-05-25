use serde::{Deserialize, Serialize};

macro_rules! flags {
    ( $( #[$enum_attrs: meta] )* pub enum $enum: ident { $(#[cfg($cfg: meta)] $(#[$attr: meta])* $flag: ident ),* $(,)?} ) => {
        $(#[$enum_attrs])*
        pub enum $enum {
            $(#[cfg($cfg)]
            $(#[$attr])*
                $flag),*
        }

        #[cfg(file_flags)]
        impl From<$enum> for nix::sys::stat::FileFlag {
            fn from(flag: $enum) -> Self {
                match flag {
                $(
                    #[cfg($cfg)]
                    $enum::$flag => nix::sys::stat::FileFlag::$flag,
                )*
                }
            }
        }
    };
}

flags! {
#[allow(non_camel_case_types)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
    Serialize,
    Deserialize,
)]
/// File flags (see https://docs.freebsd.org/en/books/handbook/basics/#permissions).
pub enum FileFlags {
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_SETTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_NODUMP,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_IMMUTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_APPEND,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    UF_OPAQUE,

    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_SETTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_ARCHIVED,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_IMMUTABLE,
    #[cfg(any(
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
    ))]
    SF_APPEND,

    #[cfg(any(target_os = "dragonfly"))]
    UF_NOHISTORY,
    #[cfg(any(target_os = "dragonfly"))]
    UF_CACHE,
    #[cfg(any(target_os = "dragonfly"))]
    UF_XLINK,
    #[cfg(any(target_os = "dragonfly"))]
    SF_NOHISTORY,
    #[cfg(any(target_os = "dragonfly"))]
    SF_CACHE,
    #[cfg(any(target_os = "dragonfly"))]
    SF_XLINK,

    #[cfg(any(target_os = "freebsd"))]
    UF_SYSTEM,
    #[cfg(any(target_os = "freebsd"))]
    UF_SPARSE,
    #[cfg(any(target_os = "freebsd"))]
    UF_OFFLINE,
    #[cfg(any(target_os = "freebsd"))]
    UF_REPARSE,
    #[cfg(any(target_os = "freebsd"))]
    UF_ARCHIVE,
    #[cfg(any(target_os = "freebsd"))]
    UF_READONLY,

    #[cfg(any(target_os = "freebsd", target_os = "netbsd"))]
    SF_SNAPSHOT,

    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    UF_NOUNLINK,
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    SF_NOUNLINK,

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
    UF_COMPRESSED,
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "watchos"))]
    UF_TRACKED,

    #[cfg(any(
        target_os = "freebsd",
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos"
    ))]
    UF_HIDDEN,

    #[cfg(any(target_os = "netbsd"))]
    SF_LOG,
    #[cfg(any(target_os = "netbsd"))]
    SF_SNAPINVAL,
}
}
