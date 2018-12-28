use clap::App;
use clap::ArgMatches;
use clap::SubCommand;
use hyper::client::connect::Connect;

use soma::Printer;
use soma::Soma;

use crate::cli::commands::SomaCommand;

pub struct ListCommand;

impl ListCommand {
    pub fn new() -> ListCommand {
        ListCommand {}
    }
}

impl SomaCommand for ListCommand {
    const NAME: &'static str = "list";

    fn app(&self) -> App<'static, 'static> {
        SubCommand::with_name(Self::NAME).about("lists docker images")
    }

    fn handle_match<C>(&self, _matches: &ArgMatches, mut soma: Soma<C>, mut printer: impl Printer)
    where
        C: 'static + Connect,
    {
        printer.write_line(&format!("{:?}", soma.list()));
    }
}
