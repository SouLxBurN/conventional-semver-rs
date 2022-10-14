# Conventional Semantic Versions

__Blazingly Fast__ 🚀 CLI tool for determining the next semantic version for a given repository based on conventional commits and git tags.

⚠️  This is in very early development. I would not recommend you integrate this tool into your project.

## Usage
```
USAGE:
    conver [OPTIONS] [PATH]

ARGS:
    <PATH>    Path to target git repository [default: .]

OPTIONS:
    -f, --bump-files    Bump the version files with the derived version
    -h, --help          Print help information
    -r, --release       Generate final release version
    -t, --tag           Tag the current commit with the release version
    -v, --lead-v        Add an optional leading v to the generated version i.e. (v2.1.3)
    -V, --version       Print version information
```

### Configuration
conventional-semver-rs will look in the root of the repositories directory for a `conventional_release.toml` file. This configuration will be applied when generating versions of updating version files.
If `conventional_release.toml` is not found, a default configuration will be loaded.

#### Default Configuration
```toml
v = false

[commit_signature]
name = "conventional-semver-rs"
email = "conventional-semver-rs@github.com"
```

#### Configuration Options
```toml
v = false # Include optional prefix v in generated version

# Customize the commit signature when bumping files and creating tags
[commit_signature]
name = "conventional-semver-rs"
email = "conventional-semver-rs@github.com"

# Describes a file containing the application's version to be updated on release
[[version_files]]
v = true # Configure option prefix v for version file
path = "version.txt" # Relative path to file
version_prefix = "" # Token to match before the version
version_postfix = "" # Token to match after the version

# Cargo.toml example
[[version_files]]
v = false
path = "Cargo.toml"
version_prefix = "version = \""
version_postfix = "\"[^,]"

# Preset example
# presets are for common version files,
# so you don't have to write the regex!
[[version_files]]
preset = "package.json"
# Currently Supported Presets
# - "Cargo.toml"
# - "package.json"
```

### Pre-release Versions
Pre-release versions are generated when the following conditions are true.
- The current commit is not tagged. (See Rebuilding Release Tags below.)
- The `--release` option _is not_ specified.
- Structure is `{MAJOR}.{MINOR}.{PATCH}-{COMMITS_SINCE_TAG}-{COMMIT_HASH}`
    - Example: `0.3.0-2-g3229751`

### Release Versions
Release versions are generated when the either of the following conditions are true.
- The current commit is already tagged with a release version.
- The `--release` option _is_ specified.
- Structure is `{MAJOR}.{MINOR}.{PATCH}`
    - Example: `0.3.0`

