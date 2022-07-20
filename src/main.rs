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

fn main() {
    let args = CmdArgs::parse();
    match conventional_semver_rs::run(&args.path, args.release) {
        Ok(version) => {
            println!("{}", version);
            if args.tag {
                match release::tag_release(&args.path, version) {
                    Ok(oid) => println!("Tag created successfully! {}", oid),
                    Err(e) => println!("Unable to tag respository: {}", e),
                }
            }
        },
        Err(e) => println!("{e}"),
    };


}

