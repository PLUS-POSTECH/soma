use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::ops::update;
use soma::prelude::*;
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};

pub struct UpdateCommand;

impl UpdateCommand {
    pub fn new() -> UpdateCommand {
        UpdateCommand {}
    }
}

impl SomaCommand for UpdateCommand {
    const NAME: &'static str = "update";

    fn app(&self) -> App {
        // TODO: update all repository when repository is omitted
        SubCommand::with_name(Self::NAME)
            .about("Updates a repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("the name of the repository to update"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        update(&env, matches.value_of("repository").unwrap())
    }
}
