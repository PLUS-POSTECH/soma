use std::fs::copy;

use clap::{Arg, ArgMatches, SubCommand};
use hyper::client::connect::Connect;

use soma::error::{Error as SomaError, Result as SomaResult};
use soma::repo::{load_manifest, MANIFEST_FILE_NAME};
use soma::{Environment, Printer};

use crate::commands::{App, SomaCommand};

pub struct DownloadCommand;

impl DownloadCommand {
    pub fn new() -> DownloadCommand {
        DownloadCommand {}
    }
}

impl SomaCommand for DownloadCommand {
    const NAME: &'static str = "download";

    fn app(&self) -> App {
        SubCommand::with_name(Self::NAME)
            .about("fetches public files from the problem")
            .arg(
                Arg::with_name("problem")
                    .required(true)
                    // TODO: Separate problem name from repository name.
                    .help("problem name with optional repository name prefix"),
            )
    }

    fn handle_match(
        &self,
        env: Environment<impl Connect + 'static, impl Printer>,
        matches: &ArgMatches,
    ) -> SomaResult<()> {
        let repo_path = env
            .data_dir()
            .repo_path()
            .join(matches.value_of("problem").unwrap());
        let repo_manifest_path = repo_path.join(MANIFEST_FILE_NAME);
        let manifest = load_manifest(repo_manifest_path)?;
        let executables = manifest.executable().iter();
        let readonly = manifest.readonly().iter();
        executables
            .chain(readonly)
            .filter(|file_entry| file_entry.public())
            .try_for_each(|file_entry| {
                let file_path = repo_path.join(file_entry.path());
                let file_name = file_path
                    .file_name()
                    .ok_or(SomaError::InvalidRepositoryPathError)?
                    .to_str()
                    .ok_or(SomaError::InvalidRepositoryPathError)?;

                env.printer()
                    .write_line(&format!("Fetching '{}'", file_name));
                copy(&file_path, file_name)?;
                Ok(())
            })
    }
}
