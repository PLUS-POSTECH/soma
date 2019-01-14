use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::ops::remove;
use soma::prelude::*;
use soma::{Environment, Printer};

use crate::commands::{default_runtime, App, SomaCommand};

pub struct RemoveCommand;

impl RemoveCommand {
    pub fn new() -> RemoveCommand {
        RemoveCommand {}
    }
}

impl SomaCommand for RemoveCommand {
    const NAME: &'static str = "remove";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Unregisters a repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("the name of the repository to remove"),
            )
    }

    fn handle_match(
        &self,
        mut env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        remove(
            &mut env,
            matches.value_of("repository").unwrap(),
            &mut default_runtime(),
        )
    }
}
