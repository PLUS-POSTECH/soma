use clap::{App, Arg, crate_version, SubCommand};
use tokio::runtime::Runtime;

fn main() {
    let matches = App::new("soma")
        .version(crate_version!())
        .about("Your one-stop CTF problem management tool")
        .subcommand(SubCommand::with_name("list")
            .about("shows the list of containers"))
        .get_matches();

    match matches.subcommand() {
        ("list", _) => {
            let mut runtime = Runtime::new().expect("failed to initialize the runtime");
            let result = runtime.block_on(soma::list());
            println!("{:#?}", result);
        }
        _ => {
            println!("no hello...");
        }
    }
}
