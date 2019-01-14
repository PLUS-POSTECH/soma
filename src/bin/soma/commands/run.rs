use clap::{value_t, Arg, ArgMatches, SubCommand};
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
            .about("Runs problem daemon container")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    .help("name of a problem"),
            )
            .arg(
                Arg::with_name("port")
                    .required(true)
                    .help("port number to run the problem"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        let port = value_t!(matches, "port", u32)?;

        run(
            &env,
            matches.value_of("problem").unwrap(),
            port,
            &mut default_runtime(),
        )?;
        Ok(())
    }
}
