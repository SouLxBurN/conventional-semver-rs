mod release;
use clap::Parser;
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

    /// Path to target git repository
    #[clap(value_parser, default_value_t = String::from("."))]
    path: String
}

fn main() -> Result<(), conventional_semver_rs::Error> {
    let args = CmdArgs::parse();
    let mut version = conventional_semver_rs::run(&args.path, args.release)?;

    let insert_v = !version.starts_with(|begin: char| begin.eq_ignore_ascii_case(&'v'));
    if args.lead_v && insert_v  {
        version.insert(0, 'v');
    }

    println!("{}", version);
    if args.tag {
        let oid = release::tag_release(&args.path, version)?;
        println!("Tag created successfully! {}", oid);
        // Err(e) => println!("Unable to tag respository: {}", e),
    }

    Ok(())
}

