use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::ops::stop;
use soma::prelude::*;
use soma::{Environment, Printer};

use crate::commands::{default_runtime, App, SomaCommand};

pub struct StopCommand;

impl StopCommand {
    pub fn new() -> StopCommand {
        StopCommand {}
    }
}

impl SomaCommand for StopCommand {
    const NAME: &'static str = "stop";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Stops a running problem")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    .help("problem name with optional repository name prefix"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        stop(
            &env,
            matches.value_of("problem").unwrap(),
            &mut default_runtime(),
        )
    }
}
