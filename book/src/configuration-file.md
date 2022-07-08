# Configuration file

The test runner can read a configuration file. For now, only the TOML format is supported.
Its path can be specified by using the `-c PATH` flag.

## Sections

### [features]

Some features are not available for every file system.
For tests requiring such features,
the execution becomes opt-in.
The user can enable their execution,
 by adding the corresponding feature as a key in this section.
A list of these opt-in features is provided
when executing the runner with `-l` argument.

For example, with `posix_fallocate`:

 ```toml
[features]
posix_fallocate = {}

# Can also be specified by using key notation
[features.posix_fallocate]
```

#### file_flags

Several UNIX operating systems have a concept of "file flags",
which adds an additional level of security and control.
Some tests are related to this feature and therefore
need it to be supported. 
However, the support of these flags is depedent of the file system,
and some file systems don't support at all any flag.
To give a sufficient level of granularity, each supported flag can be
specified in the configuration with the `file_flags` array.

```toml
[features]
posix_fallocate = {}
file_flags = ["UF_IMMUTABLE"]
```

#### Feature configuration

TODO
