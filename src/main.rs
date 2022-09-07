use clap::Parser;
use conventional_semver_rs::release;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CmdArgs {
    /// Generate final release version
    #[clap(short, long, value_parser, default_value_t = false)]
    release: bool,

    /// Tag the current commit with the release version
    #[clap(short, long, value_parser, default_value_t = false)]
    tag: bool,

    /// Add an optional leading v to the generated version i.e. (v2.1.3)
    #[clap(short='v', long, value_parser, default_value_t = false)]
    lead_v: bool,

    /// Bump the version files with the derived version
    #[clap(short='f', long, value_parser, default_value_t = false)]
    bump_files: bool,

    /// Path to target git repository
    #[clap(value_parser, default_value_t = String::from("."))]
    path: String
}

fn main() -> Result<(), conventional_semver_rs::Error> {
    let args = CmdArgs::parse();

    let mut repo = conventional_semver_rs::ConventionalRepo::new(&args.path)?;

    repo.config.v = repo.config.v || args.lead_v;
    let mut version = repo.derive_version(args.release)?;
    let insert_v = !version.starts_with(|begin: char| begin.eq_ignore_ascii_case(&'v'));
    if repo.config.v && insert_v  {
        version.insert(0, 'v');
    }

    println!("{}", version);

    let dirty = repo.is_repo_dirty()?;
    if args.bump_files && !dirty {
        let release_errors = release::bump_version_files(&args.path,
            &version,
            release::VersionFile::config_to_version_files(&repo.config));
        if release_errors.len() > 0 {
            release_errors.iter().for_each(|e| {
                eprintln!("{}", e);
            });
        }
        // TODO: Commit the version files
    }

    if args.tag && !dirty {
        let oid = release::tag_release(&repo, &version)?;
        println!("Tag created successfully! {}", oid);
    }

    Ok(())
}

