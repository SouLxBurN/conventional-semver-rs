pub mod release;
pub mod config;
extern crate custom_error;

use std::io;

use custom_error::custom_error;
use git2::{Repository, ObjectType, Oid, Revwalk, Reference};
use semver::{Prerelease, BuildMetadata};
use regex::Regex;

custom_error! { pub Error
    SemverError{source: semver::Error} = "Encountered an invalid version: {source}.",
    LSemverError{source: lenient_semver::parser::OwnedError} = "Encountered an invalid version: {source}.",
    GitError{source: git2::Error} = "Git Error: {source}",
    ConfigError{source: io::Error} = "Failed to load conventional_release.toml {source}",
    TomlError{source: toml::de::Error} = "Failed to parse conventional_release.toml {source}",
    PresetError{bad_preset: String} = "Unsupported preset found in conventional_release.toml: {bad_preset}",
    InvalidConfigError{reason: String} = "conventional_release.toml is invalid: {reason}"
}

#[derive(Clone, PartialEq, Eq)]
enum VersionBump {
    MAJOR,
    MINOR,
    PATCH
}

struct VersionBumpDetails {
    bump_type: VersionBump,
    current_version: ParsedVersion,
    rev_count: u32,
}

pub struct ConventionalRepo {
    pub config: config::ConventionalSemverConfig,
    repo: git2::Repository
}

impl ConventionalRepo {
    pub fn new(repo_path: &str) -> Result<Self, Error> where Self: Sized {
        let repo = Repository::open(repo_path)?;
        let config = config::ConventionalSemverConfig::load_config()?;
        Ok(ConventionalRepo{
            repo,
            config
        })
    }

    /// Checks if repo at `repo_path` is dirty.
    /// Returns Error result if unable to locate/open repository at `repo_path`.
    pub fn is_repo_dirty(&self) -> Result<bool, Error> {
        let mut status_options = &mut git2::StatusOptions::new();
        status_options = status_options
            .include_ignored(false)
            .include_untracked(true);
        let statuses = self.repo.statuses(Some(status_options))?;
        Ok(!statuses.is_empty())
    }

    /// If the branch head is tagged, this will return Some({version_string})
    /// Otherwise, it returns None.
    pub fn get_head_version(&self) -> Option<String> {
        let head = self.repo.head().ok()?.peel_to_commit().ok()?;
        let head_id = head.as_object().id();
        let tags = get_revision_tags(&self.repo, head_id)?;
        Some(determine_current_version(tags).original)
    }

    pub fn derive_version(&self, is_release: bool) -> Result<String, Error> {
        let dirty = self.is_repo_dirty()?;
        let head = self.repo.head()?.peel_to_commit()?;
        let head_id = head.as_object().id();
        match get_revision_tags(&self.repo, head_id) {
            Some(versions) if !dirty => {
                // Head commit is currently tagged, rebuild with highest version.
                Ok(determine_current_version(versions).original)
            },
            _ => {
                let mut version = dervive_next_version(&self.repo, head_id)?;
                // Remove Prerelease and build metadata if releasing.
                if is_release && !dirty {
                    version.parsed.pre = Prerelease::EMPTY;
                    version.parsed.build = BuildMetadata::EMPTY;
                }
                Ok(version.parsed.to_string())
            }
        }
    }
}

#[derive(Clone)]
struct ParsedVersion {
    original: String,
    parsed: semver::Version,
}

impl ParsedVersion {
    pub fn new(version: &str) -> Result<Self, Error> {
         match lenient_semver::parse(version) {
             Err(res) => Err(res.owned().into()),
             Ok(parsed) => Ok(Self{
                 original: String::from(version),
                 parsed,
             }),
         }
    }
}

/// Walks all commits and returns a prerelease version based on the commits
/// encountered between the head_id commit and the previous tag.
fn dervive_next_version(repo: &Repository, head_id: Oid) -> Result<ParsedVersion, Error> {
    let mut refs = repo.revwalk()?;
    refs.push(head_id)?;
    let details = derive_version_increase(&repo, refs)?;
    let mut version = details.current_version;
    match details.bump_type {
        VersionBump::MAJOR => {
            version.parsed.major += 1;
            version.parsed.minor = 0;
            version.parsed.patch = 0;
        },
        VersionBump::MINOR => {
            version.parsed.minor += 1;
            version.parsed.patch = 0;
        }
        _ => version.parsed.patch += 1,
    }
    version.parsed.pre = Prerelease::new(&details.rev_count.to_string()).unwrap_or_default();
    let mut oid_str = head_id.to_string();
    let build = &oid_str.as_mut_str()[..7];
    version.parsed.build = BuildMetadata::new(&build).unwrap_or_default();
    Ok(version)
}

/// Checks if the provided Oid is a tagged revision in the Repository.
/// Returns a list of all the tag names if found.
fn get_revision_tags(repo: &Repository, oid: Oid) -> Option<Vec<ParsedVersion>> {
    let reg = Regex::new(r"^.*/([vV]?\d+\.\d+\.\d+.*)$").ok()?;
    let tag_refs = repo.references_glob("refs/tags/*").ok()?;
    let tag_items: Vec<ParsedVersion> = tag_refs.filter_map(does_reference_target_commit(oid))
        .filter_map( |rev| -> Option<ParsedVersion> {
            let tag_version = reg.captures(&rev)?.get(1)?.as_str();
            let parsed = lenient_semver::parse(tag_version).ok()?;
                if parsed.pre.is_empty() && parsed.build.is_empty() {
                   return Some(ParsedVersion{
                        original: tag_version.to_string(),
                        parsed,
                    });
                }
                None
        }).collect();
    if tag_items.len() > 0 {
        return Some(tag_items)
    }
    None
}

/// Creates and returns a closure for determining if a Reference points to a given Oid/commit id.
fn does_reference_target_commit(commit_id: Oid) -> impl FnMut(Result<Reference, git2::Error>) -> Option<String> {
    move |ref_res: Result<Reference, git2::Error>| {
        let reference = ref_res.as_ref().ok()?;
        let coid = reference.peel_to_commit().ok()?;
        if coid.as_object().id() == commit_id {
            return Some(reference.name()?.to_owned());
        }
        None
    }
}

/// Determines the version bump based on the conventional commit type.
/// Crawls the repository refs from the refs HEAD to the most recent tag.
fn derive_version_increase(repo: &Repository, mut refs: Revwalk) -> Result<VersionBumpDetails, Error> {
    let mut bump_type = VersionBump::PATCH;
    let mut current_version = ParsedVersion::new("0.0.0")?;
    let mut rev_count = 0u32;

    while let Some(oid) = refs.next().transpose()? {
        if let Some(tags) = get_revision_tags(&repo, oid) {
            current_version = determine_current_version(tags);
            return Ok(VersionBumpDetails{bump_type, current_version, rev_count});
        }
        bump_type = match derive_version_from_commit(repo, oid, bump_type.clone()) {
            Some(v) => v,
            None => bump_type,
        };
        rev_count += 1;
    }
    Ok(VersionBumpDetails{bump_type, current_version, rev_count})
}

/// From a list of versions, determine the largest or most recent version
/// based on semantic verisoning.
fn determine_current_version(tags: Vec<ParsedVersion>) -> ParsedVersion {
    tags.iter().reduce(|accum: &ParsedVersion, item: &ParsedVersion| -> &ParsedVersion {
        let accum_compare = semver::Comparator{
            op: semver::Op::LessEq,
            major: accum.parsed.major,
            minor: Some(accum.parsed.minor),
            patch: Some(accum.parsed.patch),
            pre: accum.parsed.pre.clone(),
        };

        if accum_compare.matches(&item.parsed) {
            accum
        }
        else {
            item
        }
    }).expect("Unable to determine the current version").clone()
}

/// Determines the next version bump based on the commit id provided.
fn derive_version_from_commit(repo: &Repository, commit_oid: Oid, current_bump: VersionBump) -> Option<VersionBump> {
    let obj = repo.find_object(commit_oid, Some(ObjectType::Commit)).ok()?;
    let commit = obj.as_commit()?;
    let commit_msg = commit.message()?;
    let parsed_commit = git_conventional::Commit::parse(commit_msg).ok()?;
    if parsed_commit.breaking() {
        return Some(VersionBump::MAJOR);
    }
    if parsed_commit.type_() == git_conventional::Type::FEAT && current_bump == VersionBump::PATCH {
        return Some(VersionBump::MINOR);
    }
    Some(current_bump)
}
