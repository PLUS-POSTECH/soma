use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::prelude::*;
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
            .about("Builds a problem image")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    .help("the name of the problem"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        build(&env, matches.value_of("problem").unwrap())
    }
}
