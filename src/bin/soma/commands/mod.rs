use clap::ArgMatches;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;

use soma::error::Result as SomaResult;
use soma::{Environment, Printer};

pub use self::{add::AddCommand, clone::CloneCommand, list::ListCommand};

pub mod add;
pub mod clone;
pub mod list;

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

fn default_runtime() -> Runtime {
    Runtime::new().expect("failed to initialize tokio runtime")
}
