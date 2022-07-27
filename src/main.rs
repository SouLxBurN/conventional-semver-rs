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

    /// Path to target git repository
    #[clap(value_parser, default_value_t = String::from("."))]
    path: String
}

fn main() -> Result<(), conventional_semver_rs::Error> {
    let args = CmdArgs::parse();
    let version = conventional_semver_rs::run(&args.path, args.release)?;
    println!("{}", version);
    if args.tag {
        let oid = release::tag_release(&args.path, version)?;
        println!("Tag created successfully! {}", oid);
        // Err(e) => println!("Unable to tag respository: {}", e),
    }

    Ok(())
}

