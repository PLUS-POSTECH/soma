use clap::ArgMatches;
use hyper::client::connect::Connect;

use soma::Printer;
use soma::Soma;

pub mod add;
pub mod list;

type App = clap::App<'static, 'static>;

pub trait SomaCommand {
    const NAME: &'static str;

    fn app(&self) -> App;
    fn handle_match<C>(&self, matches: &ArgMatches, soma: Soma<C>, printer: impl Printer)
    where
        C: 'static + Connect;
}
