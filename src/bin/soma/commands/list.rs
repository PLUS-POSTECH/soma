use clap::ArgMatches;
use clap::SubCommand;
use hyper::client::connect::Connect;

use soma::prelude::*;
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
        SubCommand::with_name(Self::NAME).about("Lists registered repositories")
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect, impl Printer>,
        _matches: &ArgMatches,
    ) -> SomaResult<()> {
        let mut repo_iter = env.repo_manager().list_repo().peekable();

        if let None = repo_iter.peek() {
            env.printer().write_line("No repository was added.");
        } else {
            for repository in repo_iter {
                env.printer().write_line(&format!(
                    "{} ({})",
                    repository.name(),
                    repository.backend()
                ));

                let mut peekable = repository.prob_name_iter().peekable();
                while let Some(name) = peekable.next() {
                    env.printer().write_line(&format!(
                        "{}─ {}",
                        if peekable.peek().is_none() {
                            "└"
                        } else {
                            "├"
                        },
                        name
                    ))
                }
            }
        }

        Ok(())
    }
}
