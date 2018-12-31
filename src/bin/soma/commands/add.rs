use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
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
            .about("registers a soma repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("use @user/repo for GitHub repository or ./dir for local repository"),
            )
    }

    fn handle_match(
        &self,
        _env: Environment<impl Connect + 'static, impl Printer>,
        _matches: &ArgMatches,
    ) -> SomaResult<()> {
        unimplemented!()
    }
}
