use clap::ArgMatches;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;

use soma::Config;
use soma::Printer;

pub mod add;
pub mod list;

type App = clap::App<'static, 'static>;

pub trait SomaCommand {
    const NAME: &'static str;

    fn app(&self) -> App;
    fn handle_match(
        &self,
        config: Config<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    );
}

fn get_default_runtime() -> Runtime {
    Runtime::new().expect("failed to initialize tokio runtime")
}
