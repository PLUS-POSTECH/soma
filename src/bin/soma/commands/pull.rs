use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::{Error as SomaError, Result as SomaResult};
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};

pub struct PullCommand;

impl PullCommand {
    pub fn new() -> PullCommand {
        PullCommand {}
    }
}

impl SomaCommand for PullCommand {
    const NAME: &'static str = "pull";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("updates a Soma repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("name of a repository"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        let repo_name = matches.value_of("repository").unwrap();
        let repo_index = env.data_dir().read_repo_index()?;
        let repository = repo_index
            .get(repo_name)
            .ok_or(SomaError::RepositoryNotFoundError)?;
        let backend = repository.backend();
        let local_path = repository.local_path();
        backend.update(local_path)?;
        env.printer().write_line(&format!(
            "successfully updated repository: '{}'",
            &repo_name
        ));
        Ok(())
    }
}
