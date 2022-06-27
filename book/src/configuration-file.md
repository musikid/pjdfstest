# Configuration file

The test runner can read a configuration file.

## Sections

### [features]

Some syscalls cannot be run on all combinations of file systems/platforms.
Their execution is opt-in,
as in the user should enable them by adding the key in this section.
A list of these opt-in groups should be provided 
when executing the runner with `-l` argument.

```toml
[features]
posix_fallocate = true
```
