use clap::ArgMatches;
use clap::SubCommand;
use hyper::client::connect::Connect;

use soma::Config;
use soma::Printer;

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
        SubCommand::with_name(Self::NAME).about("registers a soma repository")
    }

    fn handle_match(
        &self,
        _config: Config<impl Connect + 'static, impl Printer>,
        _matches: &ArgMatches,
    ) {
        unimplemented!()
    }
}
