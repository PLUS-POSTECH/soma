use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};
use soma::ops::pull;

pub struct PullCommand;

impl PullCommand {
    pub fn new() -> PullCommand {
        PullCommand {}
    }
}

impl SomaCommand for PullCommand {
    const NAME: &'static str = "pull";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Updates a Soma repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("name of a repository"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        pull(&env, matches.value_of("repository").unwrap())
    }
}
