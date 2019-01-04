use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::ops::run;
use soma::{Environment, Printer};

use crate::commands::{default_runtime, App, SomaCommand};

pub struct RunCommand;

impl RunCommand {
    pub fn new() -> RunCommand {
        RunCommand {}
    }
}

impl SomaCommand for RunCommand {
    const NAME: &'static str = "run";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("runs problem daemon container")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    .help("name of a problem"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        run(
            &env,
            matches.value_of("problem").unwrap(),
            &mut default_runtime(),
        )
    }
}
