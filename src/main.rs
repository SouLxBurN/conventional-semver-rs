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

fn main() -> anyhow::Result<()> {
    let args = CmdArgs::parse();

    let repo = conventional_semver_rs::ConventionalRepo::new(&args.path)?;

    let mut version = repo.derive_version(args.release)?;
    if !version.starts_with(|begin: char| begin.eq_ignore_ascii_case(&'v')) {
        if repo.config.v.unwrap_or(false) || args.lead_v {
            version.insert(0, 'v');
        }
    }
    println!("{}", version);

    let dirty = repo.is_repo_dirty()?;
    let tagged_head = repo.get_head_version().is_some();
    if args.bump_files && !dirty && !tagged_head {
        let v_files = release::VersionFile::config_to_version_files(&repo.config);
        let release_errors = release::bump_version_files(&args.path,
            &version,
            &v_files);
        if release_errors.len() > 0 {
            release_errors.iter().for_each(|e| {
                eprintln!("{}", e);
            });
        }
        release::commit_version_files(&repo, &version, &v_files)?;
    }
    if (args.tag || args.bump_files) && !dirty && !tagged_head {
        release::tag_release(&repo, &version)?;
    }
    Ok(())
}

