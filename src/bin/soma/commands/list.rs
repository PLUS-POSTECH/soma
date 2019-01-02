use clap::ArgMatches;
use clap::SubCommand;
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::{Environment, Printer};

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
        SubCommand::with_name(Self::NAME).about("Lists Soma problem repositories")
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        _matches: &ArgMatches,
    ) -> SomaResult<()> {
        let repo_index = env.data_dir().read_repo_index()?;

        if repo_index.is_empty() {
            env.printer().write_line("no repository was added");
        } else {
            env.printer()
                .write_line(&format!("{:<20}{:<40}", "Name", "Origin"));

            for (repo_name, repository) in &repo_index {
                env.printer()
                    .write_line(&format!("{:<20}{:<40}", repo_name, repository.backend()));
            }
        }

        Ok(())
    }
}
