use clap::{App, Arg, crate_version, SubCommand};

fn main() {
    let matches = App::new("soma")
        .version(crate_version!())
        .about("Your one-stop CTF problem management tool")
        .subcommand(SubCommand::with_name("hello")
            .about("shows hello message"))
        .get_matches();

    match matches.subcommand() {
        ("hello", _) => {
            println!("soma hello!");
        }
        _ => {
            println!("no hello...");
        }
    }
}
