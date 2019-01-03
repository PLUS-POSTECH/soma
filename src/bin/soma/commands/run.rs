use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::{Error as SomaError, Result as SomaResult};
use soma::{Environment, Printer};
use tokio::runtime::current_thread::Runtime;

use crate::commands::{App, SomaCommand};

pub struct RunCommand;

impl RunCommand {
    pub fn new() -> RunCommand {
        RunCommand {}
    }
}

impl SomaCommand for RunCommand {
    const NAME: &'static str = "run";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("runs problem daemon container")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    .help("name of a problem"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        let repo_name = matches.value_of("problem").unwrap();
        let repo_index = env.data_dir().read_repo_index()?;
        let repository = repo_index
            .get(repo_name)
            .ok_or(SomaError::RepositoryNotFoundError)?;

        repository.build_image(&env, repo_name)?;
        env.printer().write_line(&format!(
            "successfully built image for problem: '{}'",
            &repo_name
        ));

        let mut runtime = Runtime::new().expect("failed to initialize tokio runtime");
        let conatiner_name = repository.run_container(&env, repo_name, &mut runtime)?;
        env.printer().write_line(&format!(
            "successfully started container: '{}'",
            &conatiner_name
        ));
        Ok(())
    }
}
