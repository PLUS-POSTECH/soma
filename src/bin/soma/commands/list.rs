use clap::ArgMatches;
use clap::SubCommand;
use hyper::client::connect::Connect;

use soma::docker;
use soma::Config;
use soma::Printer;

use crate::commands::get_default_runtime;
use crate::commands::{App, SomaCommand};

pub struct ListCommand;

impl ListCommand {
    pub fn new() -> ListCommand {
        ListCommand {}
    }
}

impl SomaCommand for ListCommand {
    const NAME: &'static str = "list";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME).about("lists docker images")
    }

    fn handle_match(
        &self,
        config: Config<impl Connect + 'static, impl Printer>,
        _matches: &ArgMatches,
    ) {
        let mut runtime = get_default_runtime();
        config
            .get_printer()
            .write_line(&format!("{:?}", runtime.block_on(docker::list(&config))));
    }
}
