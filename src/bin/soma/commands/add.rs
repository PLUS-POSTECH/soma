use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::ops::add;
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};

pub struct AddCommand;

impl AddCommand {
    pub fn new() -> AddCommand {
        AddCommand {}
    }
}

impl SomaCommand for AddCommand {
    const NAME: &'static str = "add";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Registers a Soma repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("git address or local path of a problem repository"),
            )
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .help("customized name for the repository")
                    .value_name("NAME")
                    .takes_value(true),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        add(
            &env,
            matches.value_of("repository").unwrap(),
            matches.value_of("NAME"),
        )
    }
}
