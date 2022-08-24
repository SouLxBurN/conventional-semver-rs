extern crate custom_error;
use custom_error::custom_error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use git2::{Signature, Time, Repository, Oid};
use regex::Regex;

custom_error! { pub Error
    VersionFileError{source: io::Error, file: String} = "Version file error({file}): {source}.",
    VersionMatchError{file: String} = "Unable find version in version file {file}",
}

const SEMVER_MATCHER: &str = r"[vV]?\d+\.\d+\.\d+[-+\w\.]*";

#[derive(Debug)]
pub struct VersionFile {
    relative_path: String,
    matcher: Regex,
}
impl VersionFile {
    pub fn new(path: String, version_prefix: String, version_postfix: String) -> Result<Self, regex::Error> {
        let regex = construct_matcher(version_prefix, version_postfix)?;
        Ok(VersionFile{
            relative_path: path,
            matcher: regex,
        })
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
pub fn bump_version_files(repo_path: &str, version: &str, files: Vec<VersionFile>) -> Vec<Error> {
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
        let cow = f.matcher.replace_all(&contents, format!("{}{}{}", cap[1].to_string(), version, cap[2].to_string()));

        // Don't write to file until files have been updated.
        // Update file
        match File::options().write(true).open(pth) {
            Ok(mut out_file) => out_file.write_all(cow.as_ref().as_bytes()).unwrap(),
            Err(e) => return Some(Error::VersionFileError{source: e, file: f.relative_path.clone()}),
        }
        None
    }).collect()
}

/// Tag Head commit of Repository as repo_path, with the provided version.
/// Handling version files TBD.
/// TODO: Should return conventional_semver::Error, don't expose the git2 lib
/// to clients.
pub fn tag_release(repo_path: &str, version: &str) -> Result<Oid, git2::Error> {
    let repo = Repository::open(&repo_path)?;
    // Tag the repository with a version
    let sig_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("We broke space and time")
        .as_millis();
    let sig = Signature::new("rs-release", "rs-release@rust.com", &Time::new(sig_time.try_into().unwrap(), 0)).unwrap();

    let head = repo.head()?.peel_to_commit().unwrap();
    repo.tag(&version.to_string(), head.as_object(), &sig, "", false)
}
