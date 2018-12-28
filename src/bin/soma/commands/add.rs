use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use hyper::client::connect::Connect;

use soma::Printer;
use soma::Soma;

use crate::commands::SomaCommand;

pub struct AddCommand;

impl AddCommand {
    pub fn new() -> AddCommand {
        AddCommand {}
    }
}

impl SomaCommand for AddCommand {
    const NAME: &'static str = "add";

    fn app(&self) -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME).about("registers a soma repository")
    }

    fn handle_match<C>(&self, _matches: &ArgMatches, mut _soma: Soma<C>, mut _printer: impl Printer)
    where
        C: 'static + Connect,
    {
        unimplemented!()
    }
}
