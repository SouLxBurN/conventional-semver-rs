extern crate custom_error;
use custom_error::custom_error;
use std::num::TryFromIntError;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use git2::{Signature, Oid};
use regex::Regex;
use git2::Commit;

use crate::config::ConventionalSemverConfig;
use crate::ConventionalRepo;

custom_error! { pub Error
    VersionFileError{source: io::Error, file: String} = "Version file error({file}): {source}.",
    VersionMatchError{file: String} = "Unable find version in version file {file}",
    SignatureError{source: TryFromIntError} = "Encountered error when attempting to create git signature timpstamp {source}",
    GitError{source: git2::Error} = "An error occurred when performing a Git action: {source}",
}

static SEMVER_MATCHER: &str = r"[vV]?\d+\.\d+\.\d+[-+\w\.]*";

#[derive(Debug)]
pub struct VersionFile {
    relative_path: String,
    matcher: Regex,
    v: bool,
}
impl VersionFile {
    pub fn new(path: String, version_prefix: String, version_postfix: String, v: bool) -> Result<Self, regex::Error> {
        let regex = construct_matcher(version_prefix, version_postfix)?;
        Ok(VersionFile{
            relative_path: path,
            matcher: regex,
            v,
        })
    }

    pub fn config_to_version_files(config: &ConventionalSemverConfig) -> anyhow::Result<Vec<VersionFile>> {
        match &config.version_files {
            None => Ok(vec![]),
            Some(version_files) => {
                version_files.iter().map(|v_file| -> anyhow::Result<VersionFile> {
                    Ok(VersionFile::new(
                        v_file.path.clone(),
                        v_file.version_prefix.as_ref()
                            .unwrap_or(&String::from("")).clone(),
                        v_file.version_postfix.as_ref()
                            .unwrap_or(&String::from("")).clone(),
                        v_file.v
                    )?)
                }).collect()
            }
        }
    }
}

/// Compiles the provided prefix and postfix into a Regex with the SEMVER_MATCHER constant
/// Example: `version_prefix: "version = \\""`, `version_postfix: "\\"[^,]"`
/// Compiled: `(version = \\"){SEMVER_MATCHER}(\\"[^,])`
/// Matches: `version = "2.12.18"`
fn construct_matcher(prefix: String, postfix: String) -> Result<regex::Regex, regex::Error> {
    Ok(Regex::new(&format!("({}){}({})", prefix, SEMVER_MATCHER, postfix))?)
}

/// Update versions in various version files.
/// package.josn, cargo.toml, etc.
pub fn bump_version_files(repo_path: &str, version: &str, files: &Vec<VersionFile>) -> Vec<Error> {
    let version = match version.strip_prefix("v") {
        Some(v) => v,
        None => version,
    };

    files.iter().filter_map(|f| -> Option<Error> {
        // Get file based on relative path
        let str_pth = format!("{}/{}", repo_path, f.relative_path).to_string();
        let pth = Path::new(&str_pth);
        let contents = match std::fs::read_to_string(pth) {
            Ok(c) => c,
            Err(e) => return Some(Error::VersionFileError{source: e, file: f.relative_path.clone()}),
        };

        // Scan file contents with matcher regex
        let cap = match f.matcher.captures(&contents) {
            Some(c) => c,
            None => return Some(Error::VersionMatchError{file: f.relative_path.clone()}),
        };

        let fmt_str = match f.v {
            true => format!("{}v{}{}", cap[1].to_string(), version, cap[2].to_string()),
            false => format!("{}{}{}", cap[1].to_string(), version, cap[2].to_string()),
        };
        let cow = f.matcher.replace_all(&contents, fmt_str);

        // Don't write to file until files have been updated.
        // Update file
        match File::options().write(true).open(pth) {
            Ok(mut out_file) => {
                out_file.write_all(cow.as_ref().as_bytes()).err()?;
            },
            Err(e) => return Some(Error::VersionFileError{source: e, file: f.relative_path.clone()}),
        }
        None
    }).collect()
}

/// Tag Head commit of Repository, with the provided version.
pub fn tag_release(repo: &ConventionalRepo, version: &str) -> Result<Oid, Error> {
    // Tag the repository with a version
    let sig = Signature::now(
        &repo.config.commit_signature.name,
        &repo.config.commit_signature.email)?;
    let head = repo.repo.head()?.peel_to_commit()?;
    Ok(repo.repo.tag(&version.to_string(), head.as_object(), &sig, "", false)?)
}

pub fn commit_version_files(
    repo: &ConventionalRepo,
    version: &str,
    version_files: &Vec<VersionFile>
) -> Result<Oid, Error> {
    let sig = Signature::now(
        &repo.config.commit_signature.name,
        &repo.config.commit_signature.email)?;

    let head = repo.repo.head()?;
    let commit = head.peel_to_commit()?;
    let parent_commits: [&Commit; 1] = [&commit];

    let mut index = repo.repo.index()?;
    version_files.iter().for_each(|v: &VersionFile| {
        if let Err(e) = index.add_path(&Path::new(&v.relative_path)) {
            eprintln!("Error Encountered {}", e);
        }
    });
    index.write()?;

    // Regrab index from repo, to prevent staging old changes.
    let mut index = repo.repo.index()?;
    let oid = index.write_tree()?;
    let commit_tree = repo.repo.find_tree(oid)?;

    Ok(repo.repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &format!("chore(release): created release {}", version).to_owned(),
        &commit_tree,
        &parent_commits
    )?)
}

