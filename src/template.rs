use std::fs::File;
use std::path::Path;

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

use crate::error::Result as SomaResult;
use crate::repo::Manifest;
use crate::VERSION;

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
pub struct RenderingContext<'a> {
    username: &'a str,
    version: &'a str,
    repository_name: &'a str,
    manifest: Manifest,
}

impl<'a> RenderingContext<'a> {
    pub fn new(
        username: &'a String,
        repository_name: &'a str,
        manifest: Manifest,
    ) -> RenderingContext<'a> {
        RenderingContext {
            username,
            version: VERSION,
            repository_name,
            manifest,
        }
    }
}

pub trait HandleBarsExt {
    fn render_templates(
        &self,
        templates: Templates,
        input_data: &impl Serialize,
        output_path: impl AsRef<Path>,
    ) -> SomaResult<()>;
}

impl HandleBarsExt for Handlebars {
    fn render_templates(
        &self,
        templates: Templates,
        input_data: &impl Serialize,
        output_path: impl AsRef<Path>,
    ) -> SomaResult<()> {
        for (file_name, template_string) in templates.templates() {
            let mut rendered_file = File::create(output_path.as_ref().join(file_name))?;
            self.render_template_to_write(template_string, &input_data, &mut rendered_file)?;
        }
        Ok(())
    }
}
