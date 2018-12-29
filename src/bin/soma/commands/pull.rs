use std::fs::File;
use std::path::Path;

use clap::{Arg, ArgMatches, SubCommand};
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use git2::Repository;
use handlebars::Handlebars;
use hyper::client::connect::Connect;
use serde_derive::{Deserialize, Serialize};
use tempfile::tempdir;

use soma::docker;
use soma::error::{Error as SomaError, Result as SomaResult};
use soma::repo::Repository as SomaRepository;
use soma::Environment;
use soma::Printer;
use soma::SomaInfo;

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
            .about("update a soma repository and build soma image")
            .arg(Arg::with_name("repo").required(true))
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

#[derive(Deserialize, Serialize)]
struct RenderingInput {
    soma_info: SomaInfo,
    repository: SomaRepository,
}

pub fn build_soma_image(
    env: Environment<impl Connect + 'static, impl Printer>,
    image_name: &str,
    repo_path: impl AsRef<Path>,
    repository: SomaRepository,
) -> SomaResult<()> {
    let temp_dir = tempdir()?;
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy_items(&vec![&repo_path], &temp_dir, &copy_options)?;

    let binary_templates_path = env.data_dir().templates_path().join("binary");
    let repo_dir_name = repo_path
        .as_ref()
        .file_name()
        .ok_or(SomaError::InvalidRepositoryError)?;
    let work_dir = temp_dir.path().join(repo_dir_name);

    let render_engine = Handlebars::new();
    let rendering_input = RenderingInput {
        soma_info: env.soma_info().clone(),
        repository,
    };

    let mut template_docker_file = File::open(binary_templates_path.join("Dockerfile"))?;
    let mut rendered_docker_file = File::create(work_dir.join("Dockerfile"))?;
    render_engine.render_template_source_to_write(
        &mut template_docker_file,
        &rendering_input,
        &mut rendered_docker_file,
    )?;

    let mut template_entry_file = File::open(binary_templates_path.join("start.sh"))?;
    let mut rendered_entry_file = File::create(work_dir.join("start.sh"))?;
    render_engine.render_template_source_to_write(
        &mut template_entry_file,
        &rendering_input,
        &mut rendered_entry_file,
    )?;

    docker::build(image_name, work_dir)?;
    temp_dir.close()?;
    Ok(())
}
