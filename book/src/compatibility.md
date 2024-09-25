# Compatibility

The test suite is designed to be compatible with multiple operating systems and file systems.
It is primarily focused on POSIX compliance for FreeBSD file systems, but is also compatible with other operating systems and file systems
in addition of supplementary tests.

## Supported operating systems

### Active support

The test suite has been tested and is actively maintained on the following operating systems:

- FreeBSD (main development platform)
  - UFS
  - ZFS
- Linux
  - ext4

### Experimental support

The test suite has not been tested on the following operating systems, but should work:

- MacOS
- NetBSD
- OpenBSD
- Solaris
- DragonFly BSD
- Illumos
- Android
- iOS

## Missing tests

The test suite is a complete rewrite of the original test suite in Rust. Many tests have been ported, but some tests are still missing.

<!-- cmdrun python3 ../compatibility_report.py -->
