use std::time::{SystemTime, UNIX_EPOCH};
use git2::{Signature, Time, Repository, Error, Oid};

/// Tag Head commit of Repository as repo_path, with the provided version.
/// Handling version files TBD.
pub fn tag_release(repo_path: &str, version: String) -> Result<Oid, Error> {
    let repo = Repository::open(&repo_path)?;
    // Tag the repository with a version
    let sig_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("We broke space and time")
        .as_millis();
    let sig = Signature::new("rs-release", "rs-release@rust.com", &Time::new(sig_time.try_into().unwrap(), 0)).unwrap();

    let head = repo.head()?.peel_to_commit().unwrap();
    return repo.tag(&version.to_string(), head.as_object(), &sig, "", false)
}
