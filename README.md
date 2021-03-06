# Conventional Semantic Versions

__Blazingly Fast__ 🚀 CLI tool for determining the next semantic version for a given repository based on conventional commits and git tags.

⚠️  This is in very early development. I would not recommend you integrate this tool into your project.

## Usage
```
USAGE:
    conventional-semver-rs [OPTIONS] <PATH>

ARGS:
    <PATH>    Path to target git repository [default: .]

OPTIONS:
    -h, --help       Print help information
    -r, --release    Generate final release version
    -t, --tag        Tag the current commit with the release version
    -V, --version    Print version information
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

