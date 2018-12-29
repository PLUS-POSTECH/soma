use clap::{Arg, ArgMatches, SubCommand};
use git2::Repository;
use hyper::client::connect::Connect;
use tempfile::tempdir;

use soma::error::Result as SomaResult;
use soma::Environment;
use soma::Printer;

use crate::commands::{App, SomaCommand};

pub struct CloneCommand;

impl CloneCommand {
    pub fn new() -> CloneCommand {
        CloneCommand {}
    }
}

impl SomaCommand for CloneCommand {
    const NAME: &'static str = "clone";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("clone a git repository to the cache directory")
            .arg(Arg::with_name("url").required(true))
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        let dir = tempdir()?;
        let repo = Repository::clone(matches.value_of("url").unwrap(), dir.path())?;
        env.printer().write_line(&format!(
            "{:?}",
            repo.remotes()?
                .iter()
                .filter_map(|str| str)
                .collect::<Vec<&str>>()
        ));

        env.printer().write_line(&format!("{:?}", dir.path()));

        Ok(())
    }
}
