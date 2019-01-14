use clap::ArgMatches;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;

use soma::error::Result as SomaResult;
use soma::{Environment, Printer};

pub use self::{
    add::AddCommand, build::BuildCommand, fetch::FetchCommand, list::ListCommand,
    remove::RemoveCommand, run::RunCommand,
};

pub mod add;
pub mod build;
pub mod fetch;
pub mod list;
pub mod remove;
pub mod run;

type App = clap::App<'static, 'static>;

pub trait SomaCommand {
    const NAME: &'static str;

    fn app(&self) -> App;
    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()>;
}

fn default_runtime() -> Runtime {
    Runtime::new().expect("Failed to initialize tokio runtime")
}
