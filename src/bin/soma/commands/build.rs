use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};
use soma::ops::build;

pub struct BuildCommand;

impl BuildCommand {
    pub fn new() -> BuildCommand {
        BuildCommand {}
    }
}

impl SomaCommand for BuildCommand {
    const NAME: &'static str = "build";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Builds Soma image")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    .help("name of the problem"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        build(&env, matches.value_of("problem").unwrap())
    }
}
