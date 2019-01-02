use std::fs::File;
use std::path::Path;

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use handlebars::Handlebars;
use hyper::client::connect::Connect;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use crate::docker;
use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::{load_manifest, Manifest, MANIFEST_FILE_NAME};
use crate::{Environment, Printer, VERSION};

enum Templates {
    Binary,
}

impl Templates {
    fn templates(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Templates::Binary => vec![
                ("Dockerfile", include_str!("../templates/binary/Dockerfile")),
                ("start.sh", include_str!("../templates/binary/start.sh")),
            ],
        }
    }
}

#[derive(Deserialize, Serialize)]
struct RenderingInput<'a> {
    username: &'a str,
    version: &'a str,
    repository_name: &'a str,
    manifest: Manifest,
}

pub fn build_soma_image(
    env: &Environment<impl Connect + 'static, impl Printer>,
    image_name: &str,
    repo_path: impl AsRef<Path>,
) -> SomaResult<()> {
    let temp_dir = tempdir()?;
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy_items(&vec![&repo_path], &temp_dir, &copy_options)?;

    let repository_name = repo_path
        .as_ref()
        .file_name()
        .ok_or(SomaError::InvalidRepositoryPathError)?
        .to_str()
        .ok_or(SomaError::InvalidRepositoryPathError)?;
    let work_dir = temp_dir.path().join(repository_name);
    let manifest = load_manifest(work_dir.join(MANIFEST_FILE_NAME))?;

    let rendering_input = RenderingInput {
        username: env.username(),
        version: VERSION,
        repository_name,
        manifest,
    };

    Templates::Binary.templates().into_iter().try_for_each(
        |(file_name, template_string)| -> SomaResult<()> {
            render_file_from_template_string(
                template_string,
                &rendering_input,
                work_dir.join(file_name),
            )?;
            Ok(())
        },
    )?;

    docker::build(image_name, work_dir)?;
    temp_dir.close()?;
    Ok(())
}

fn render_file_from_template_string<T>(
    template_string: &str,
    input_data: &T,
    output_path: impl AsRef<Path>,
) -> SomaResult<()>
where
    T: Serialize,
{
    let render_engine = Handlebars::new();
    let mut rendered_file = File::create(output_path)?;
    render_engine.render_template_to_write(template_string, &input_data, &mut rendered_file)?;
    Ok(())
}
