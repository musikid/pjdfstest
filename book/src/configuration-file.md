# Configuration file

The test runner can read a configuration file. For now, only the TOML format is supported.
Its path can be specified by using the `-c PATH` flag.

## Sections

### [features]

Some syscalls cannot be run on all combinations of file systems/platforms.
Their execution is opt-in,
as in the user should enable them by adding the key in this section.
A list of these opt-in groups should be provided 
when executing the runner with `-l` argument.

```toml
[features]
posix_fallocate = {}

# Can also be specified by using key notation
[features.posix_fallocate]
```

#### Feature configuration

TODO
