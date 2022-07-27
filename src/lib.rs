extern crate custom_error;
use custom_error::custom_error;
use conventional_commits_parser::parse_commit_msg;
use git2::{Repository, DescribeOptions, DescribeFormatOptions, ObjectType, Oid, Revwalk, Reference};
use semver::{Version, Prerelease, BuildMetadata};

custom_error! { pub Error
    SemverError{source: semver::Error} = "Encountered an invalid version: {source}.",
    GitError{source: git2::Error} = "Git Error: {source}"
}

#[derive(Clone, PartialEq, Eq)]
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

pub fn run(repo_path: &str, is_release: bool) -> Result<String, Error> {
    let repo = Repository::open(&repo_path)?;
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
        if is_release {
            version.pre = Prerelease::EMPTY;
            version.build = BuildMetadata::EMPTY;
        }
    }
    Ok(version.to_string())
}

/// Checks if the provided Oid is a tagged revision in the Repository.
/// Returns the name of the tag if found.
fn get_revision_tag(repo: &Repository, oid: Oid) -> Option<String> {
    let mut tag_refs = repo.references_glob("refs/tags/*").ok()?;
    let tag_item = tag_refs.find(does_reference_target_commit(oid))?;
    let id = tag_item.ok()?;
    let t_name = id.name()?;
    Some(t_name.to_owned())
}

/// Creates and returns a closure for determining if a Reference points to a given Oid/commit id.
fn does_reference_target_commit(commit_id: Oid) -> impl FnMut(&Result<Reference, git2::Error>) -> bool {
    move |t_ref: &Result<Reference, git2::Error>| {
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
        bump_type = match derive_version_from_commit(repo, oid, bump_type.clone()) {
            Some(v) => v,
            None => bump_type,
        };
        rev_count += 1;
    }
    Ok(VersionBumpDetails{bump_type, rev_count})
}

///
fn derive_version_from_commit(repo: &Repository, oid: Oid, current_bump: VersionBump) -> Option<VersionBump> {
    let obj = repo.find_object(oid, Some(ObjectType::Commit)).ok()?;
    let commit = obj.as_commit()?;
    let commit_msg = commit.message()?;
    let parsed_commit = parse_commit_msg(commit_msg).ok()?;
    if parsed_commit.is_breaking_change {
        return Some(VersionBump::MAJOR);
    }
    if parsed_commit.ty == COMMIT_TYPE_FEAT && current_bump == VersionBump::PATCH {
        return Some(VersionBump::MINOR);
    }
    Some(current_bump)
}
