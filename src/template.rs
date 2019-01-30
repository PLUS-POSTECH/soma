use std::fs::File;
use std::path::Path;

use handlebars::Handlebars;
use serde::Serialize;

use crate::prelude::*;
use crate::repo::SolidManifest;
use crate::VERSION;

pub enum Templates {
    Binary,
}

impl Templates {
    fn templates(&self) -> &[(&str, &str)] {
        match self {
            Templates::Binary => &[
                ("Dockerfile", include_str!("../templates/binary/Dockerfile")),
                ("start.sh", include_str!("../templates/binary/start.sh")),
            ],
        }
    }
}

// TODO: Problem name in rendering context
#[derive(Serialize)]
pub struct RenderingContext<'a> {
    username: &'a str,
    version: &'a str,
    repository_name: &'a str,
    manifest: SolidManifest,
}

impl<'a> RenderingContext<'a> {
    pub fn new(
        username: &'a String,
        repository_name: &'a str,
        manifest: SolidManifest,
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
        context: &impl Serialize,
        output_dir: impl AsRef<Path>,
    ) -> SomaResult<()>;
}

impl HandleBarsExt for Handlebars {
    fn render_templates(
        &self,
        templates: Templates,
        context: &impl Serialize,
        output_dir: impl AsRef<Path>,
    ) -> SomaResult<()> {
        for (file_name, template_string) in templates.templates() {
            let mut rendered_file = File::create(output_dir.as_ref().join(file_name))?;
            self.render_template_to_write(template_string, context, &mut rendered_file)?;
        }

        Ok(())
    }
}
