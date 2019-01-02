use std::fs::File;
use std::path::Path;

use handlebars::Handlebars;
use hyper::client::connect::Connect;
use serde::{Deserialize, Serialize};

use crate::error::Result as SomaResult;
use crate::repo::Manifest;
use crate::{Environment, Printer, VERSION};

pub enum Templates {
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
pub struct RenderingInput<'a> {
    username: &'a str,
    version: &'a str,
    repository_name: &'a str,
    manifest: Manifest,
}

impl<'a> RenderingInput<'a> {
    pub fn new(
        env: &'a Environment<impl Connect + 'static, impl Printer>,
        repository_name: &'a str,
        manifest: Manifest,
    ) -> RenderingInput<'a> {
        RenderingInput {
            username: env.username(),
            version: VERSION,
            repository_name,
            manifest,
        }
    }
}

pub fn render_files_from_template<T>(
    template: Templates,
    input_data: &T,
    output_path: impl AsRef<Path>,
) -> SomaResult<()>
where
    T: Serialize,
{
    template.templates().into_iter().try_for_each(
        |(file_name, template_string)| -> SomaResult<()> {
            let render_engine = Handlebars::new();
            let mut rendered_file = File::create(output_path.as_ref().join(file_name))?;
            Ok(render_engine.render_template_to_write(
                template_string,
                &input_data,
                &mut rendered_file,
            )?)
        },
    )
}
