use std::env::current_dir;

use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::ops::fetch;
use soma::prelude::*;
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};

pub struct FetchCommand;

impl FetchCommand {
    pub fn new() -> FetchCommand {
        FetchCommand {}
    }
}

impl SomaCommand for FetchCommand {
    const NAME: &'static str = "fetch";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Fetches public files from the problem")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    // TODO: Separate problem name from repository name.
                    .help("problem name with optional repository name prefix"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        fetch(&env, matches.value_of("problem").unwrap(), current_dir()?)
    }
}
