use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::Result as SomaResult;
use soma::ops::parse_repo_url;
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};

pub struct AddCommand;

impl AddCommand {
    pub fn new() -> AddCommand {
        AddCommand {}
    }
}

impl SomaCommand for AddCommand {
    const NAME: &'static str = "add";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("Registers a Soma repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("git address or local path of a problem repository"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        let (repo_name, backend) = parse_repo_url(matches.value_of("repository").unwrap())?;
        env.data_dir().add_repo(repo_name.clone(), backend)?;
        env.printer()
            .write_line(&format!("successfully added a repository '{}'", &repo_name));
        Ok(())
    }
}
