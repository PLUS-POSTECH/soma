use std::fs::File;
use std::path::Path;

use handlebars::Handlebars;
use serde::Serialize;

use crate::prelude::*;

pub enum Templates {
    Binary,
}

impl Templates {
    fn templates(&self) -> &[(&str, &str)] {
        match self {
            Templates::Binary => &[
                ("Dockerfile", include_str!("../templates/binary/Dockerfile")),
                (
                    ".soma/start.sh",
                    include_str!("../templates/binary/start.sh"),
                ),
                (
                    ".soma/configure_permissions.sh",
                    include_str!("../templates/binary/configure_permissions.sh"),
                ),
            ],
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
