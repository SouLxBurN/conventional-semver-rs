use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use conventional_commits_parser::parse_commit_msg;
use git2::{Error, Repository, DescribeOptions, DescribeFormatOptions, ObjectType, Oid, Revwalk, Reference, Signature, Time};
use semver::{Version, Prerelease, BuildMetadata};

#[derive(PartialEq, Eq)]
enum VersionBump {
    MAJOR,
    MINOR,
    PATCH
}

struct VersionBumpDetails {
    bump_type: VersionBump,
    rev_count: u32,
}

const COMMIT_TYPE_FEAT: &str = "feat";

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CmdArgs {
    /// Generate final release version
    #[clap(short, long, value_parser, default_value_t = false)]
    release: bool,

    /// Tag the current commit with the release version
    #[clap(short, long, value_parser, default_value_t = false)]
    tag: bool,

    /// Path to target git repository
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
        Ok(describe) => describe.format(Some(&DescribeFormatOptions::new()))?,
        Err(_) => { String::from("0.0.0") },
    };

    let mut version = match Version::parse(&v_input) {
        Ok(v) => v,
        Err(e) => panic!("Unable to parse {} as valid semver: {}", v_input, e)
    };

    // Zero commits. // Not supported.
    let head = repo.head()?.peel_to_commit().unwrap();
    let head_id = head.as_object().id();
    let rev_tag = get_revision_tag(&repo, head_id);
    if rev_tag.is_none() {
        let mut refs = repo.revwalk()?;
        refs.push(head_id)?;

        let v_result = derive_version_increase(&repo, refs);
        match v_result {
            Ok(details) => {
                match details.bump_type {
                    VersionBump::MAJOR => {
                        version.major += 1;
                        version.minor = 0;
                        version.patch = 0;
                    },
                    VersionBump::MINOR => {
                        version.minor += 1;
                        version.patch = 0;
                    }
                    _ => version.patch += 1,
                }
                version.pre = Prerelease::new(&details.rev_count.to_string()).unwrap_or_default();
                let mut oid_str = head_id.to_string();
                let build = &oid_str.as_mut_str()[..7];
                version.build = BuildMetadata::new(&build).unwrap_or_default();
            },
            Err(e) => {
                version.patch += 1;
                println!("Encountered an error when deriving version increase: {}", e);
            },
        }

        // Remove Prerelease and build metadata if releasing.
        if args.release {
            version.pre = Prerelease::EMPTY;
            version.build = BuildMetadata::EMPTY;
        }
    }

    if args.tag {
        // Tag the repository with a version
        let sig_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("We broke space and time")
            .as_millis();
        let sig = Signature::new("rs-release", "rs-release@rust.com", &Time::new(sig_time.try_into().unwrap(), 0)).unwrap();
        match repo.tag(&version.to_string(), head.as_object(), &sig, "", false) {
            Ok(oid) => println!("Tag created successfully! {}", oid),
            Err(e) => println!("Unable to tag respository: {}", e),
        }
    }

    println!("{}", version);
    Ok(())
}

/// Checks if the provided Oid is a tagged revision in the Repository.
/// Returns the name of the tag if found.
fn get_revision_tag(repo: &Repository, oid: Oid) -> Option<String> {
    if let Ok(mut tag_refs) = repo.references_glob("refs/tags/*") {
        match tag_refs.find(does_reference_target_commit(oid)) {
            Some(tag_item) => {
                if let Ok(id) = tag_item {
                    if let Some(t_name) = id.name() {
                        return Some(t_name.to_owned());
                    }
                }
            },
            None => return None,
        }
    }
    None
}

/// Creates and returns a closure for determining if a Reference points to a given Oid/commit id.
fn does_reference_target_commit(commit_id: Oid) -> impl FnMut(&Result<Reference, Error>) -> bool {
    move |t_ref: &Result<Reference, Error>| {
        if let Ok(id) = t_ref {
            if let Some(coid) = id.peel_to_commit().ok() {
                if coid.as_object().id() == commit_id {
                    return true;
                }
            }
        }
        false
    }
}

/// Determines the version bump based on the conventional commit type.
/// Crawls the repository refs from the refs HEAD to the most recent tag.
fn derive_version_increase(repo: &Repository, mut refs: Revwalk) -> Result<VersionBumpDetails, Error> {
    let mut bump_type = VersionBump::PATCH;
    let mut rev_count = 0u32;

    while let Some(oid) = refs.next().transpose()? {
        if let Some(_) = get_revision_tag(&repo, oid) {
            return Ok(VersionBumpDetails{bump_type, rev_count});
        }
        let obj = repo.find_object(oid, Some(ObjectType::Commit))?;
        if let Some(commit) = obj.as_commit() {
            if let Some(commit_msg) = commit.message() {
                if let Ok(parsed_commit) = parse_commit_msg(commit_msg) {
                    if parsed_commit.is_breaking_change {
                        bump_type = VersionBump::MAJOR;
                    }
                    if parsed_commit.ty == COMMIT_TYPE_FEAT && bump_type == VersionBump::PATCH {
                        bump_type = VersionBump::MINOR;
                    }
                }
            }
        }
        rev_count += 1;
    }
    return Ok(VersionBumpDetails{bump_type, rev_count});
}

