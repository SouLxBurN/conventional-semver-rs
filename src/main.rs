use clap::Parser;
use conventional_commits_parser::parse_commit_msg;
use git2::{Error, Repository, DescribeOptions, DescribeFormatOptions, ObjectType, Oid, Revwalk};
use semver::{Version, Prerelease, BuildMetadata};

#[derive(PartialEq, Eq)]
enum VersionBump {
    MAJOR,
    MINOR,
    PATCH
}

const COMMIT_TYPE_FEAT: &str = "feat";

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CmdArgs {
    #[clap(short, long, value_parser, default_value_t = false)]
    release: bool,
    #[clap(value_parser, default_value_t = String::from("."))]
    path: String
}

fn main() {
    let args = CmdArgs::parse();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("{e}"),
    };
}

fn run(args: &CmdArgs) -> Result<(), Error> {
    let repo = Repository::open(&args.path)?;
    let v_input = match repo.describe(DescribeOptions::new().describe_tags()) {
        Ok(describe) => describe.format(Some(DescribeFormatOptions::new().dirty_suffix("-dirty")))?,
        Err(_) => { String::from("0.1.0") },
    };

    let mut version = match Version::parse(&v_input) {
        Ok(v) => v,
        Err(e) => panic!("Unable to parse {} as valid semver: {}", v_input, e)
    };

    // TODO: Handle Zero Case
    // Zero commits. // Not supported.
    // Zero tags. // Have revwalk and count commits.
    if !version.pre.is_empty() {
        let mut refs = repo.revwalk()?;
        let head = repo.head()?.peel_to_commit().unwrap();
        refs.push(head.as_object().id())?;

        match derive_version_increase(&repo, refs) {
            Ok(VersionBump::MAJOR) => {
                version.major += 1;
                version.minor = 0;
                version.patch = 0;
            },
            Ok(VersionBump::MINOR) => {
                version.minor += 1;
                version.patch = 0;
            },
            _ => version.patch += 1,
        }

        // Remove Prerelease and build metadata if releasing.
        if args.release {
            version.pre = Prerelease::EMPTY;
            version.build = BuildMetadata::EMPTY;
        }
    }

    println!("{}", version);
    Ok(())
}

/// Checks if the provided Oid is a tagged revision in the Repository.
fn is_tagged_rev(repo: &Repository, oid: Oid) -> bool {
    if let Ok(mut tag_refs) = repo.references_glob("refs/tags/*") {
        if tag_refs.any(|t_ref| {
            if let Ok(id) = t_ref {
                if let Some(coid) = id.target() {
                    if coid == oid {
                        return true;
                    }
                }
            }
            false
        }) {
            return true;
        }
    }
    false
}

/// Determines the version bump based on the conventional commit type.
/// Crawls the repository refs from the refs HEAD to the most recent tag.
fn derive_version_increase(repo: &Repository, mut refs: Revwalk) -> Result<VersionBump, Error> {
    let mut inc = VersionBump::PATCH;

    while let Some(oid) = refs.next().transpose()? {
        if is_tagged_rev(&repo, oid) {
            return Ok(inc);
        }
        let obj = repo.find_object(oid, Some(ObjectType::Commit))?;
        if let Some(commit) = obj.as_commit() {
            if let Some(commit_msg) = commit.message() {
                if let Ok(parsed_commit) = parse_commit_msg(commit_msg) {
                    if parsed_commit.is_breaking_change {
                        inc = VersionBump::MAJOR;
                    }
                    if parsed_commit.ty == COMMIT_TYPE_FEAT && inc == VersionBump::PATCH {
                        inc = VersionBump::MINOR;
                    }
                }
            }
        }
    }
    return Ok(inc);
}
