use std::path::Path;

use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;
use url::Url;

use soma::error::{Error as SomaError, Result as SomaResult};
use soma::repo::backend::Backend;
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
            .about("registers a soma repository")
            .arg(
                Arg::with_name("repository")
                    .required(true)
                    .help("git address or local path of a problem repository"),
            )
    }

    fn handle_match(
        &self,
        _env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        println!(
            "{:?}",
            parse_repo_url(matches.value_of("repository").unwrap())
        );
        Ok(())
    }
}

fn parse_repo_url(url: &str) -> SomaResult<(String, Backend)> {
    let path = Path::new(url);
    if path.is_dir() {
        // local backend
        Ok((
            format!(
                "#{}",
                path.file_name()
                    .ok_or(SomaError::InvalidRepositoryError)?
                    .to_str()
                    .ok_or(SomaError::InvalidRepositoryError)?
            ),
            Backend::LocalBackend(path.canonicalize()?.to_owned()),
        ))
    } else {
        // git backend
        let parsed_url = Url::parse(url)?;
        let last_name = parsed_url
            .path_segments()
            .ok_or(SomaError::InvalidRepositoryError)?
            .last()
            .ok_or(SomaError::InvalidRepositoryError)?;
        let repo_name = if last_name.ends_with(".git") {
            &last_name[..last_name.len() - 4]
        } else {
            &last_name
        };
        Ok((repo_name.to_owned(), Backend::GitBackend(url.to_owned())))
    }
}
