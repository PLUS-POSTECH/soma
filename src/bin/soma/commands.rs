use clap::ArgMatches;
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::{Environment, Printer};

pub use self::{
    add::AddCommand, fetch::FetchCommand, list::ListCommand, pull::PullCommand, run::RunCommand,
};

pub mod add;
pub mod fetch;
pub mod list;
pub mod pull;
pub mod run;

type App = clap::App<'static, 'static>;

pub trait SomaCommand {
    const NAME: &'static str;

    fn app(&self) -> App;
    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()>;
}
