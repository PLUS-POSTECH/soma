use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::image::{APIImages, CreateImageOptions, CreateImageResults, ListImagesOptions};
use failure::Error;
use futures::stream::Stream;
use futures::Future;
use hyper::client::connect::Connect;

use crate::error::Result as SomaResult;
use crate::{Environment, Printer, VERSION};

pub const LABEL_KEY_VERSION: &'static str = "soma.version";
pub const LABEL_KEY_USERNAME: &'static str = "soma.username";
pub const LABEL_KEY_REPOSITORY: &'static str = "soma.repository";

#[derive(Debug)]
pub enum SomaAPIImages {
    Normal(APIImages),
    VersionMismatch(APIImages),
    NoVersionFound(APIImages),
}

impl SomaAPIImages {
    pub fn api_images(&self) -> &APIImages {
        match self {
            SomaAPIImages::Normal(image) => image,
            SomaAPIImages::VersionMismatch(image) => image,
            SomaAPIImages::NoVersionFound(image) => image,
        }
    }
}

pub fn list(
    env: &Environment<impl Connect + 'static, impl Printer>,
) -> impl Future<Item = Vec<SomaAPIImages>, Error = Error> {
    let username = env.username().clone();
    env.docker
        .list_images(Some(ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .map(move |images| -> Vec<SomaAPIImages> {
            images
                .into_iter()
                .filter(|image| match &image.labels {
                    Some(labels) => match labels.get(LABEL_KEY_USERNAME) {
                        Some(image_username) => image_username == &username,
                        None => false,
                    },
                    None => false,
                })
                .map(|image| match &image.labels {
                    Some(labels) => match labels.get(LABEL_KEY_VERSION) {
                        Some(image_version) => {
                            if image_version == VERSION {
                                SomaAPIImages::Normal(image)
                            } else {
                                SomaAPIImages::VersionMismatch(image)
                            }
                        }
                        None => SomaAPIImages::NoVersionFound(image),
                    },
                    None => panic!("Impossible"),
                })
                .collect()
        })
}

pub fn pull<'a>(
    env: &Environment<impl Connect + 'static, impl Printer>,
    image_name: &'a str,
) -> impl Future<Item = Vec<CreateImageResults>, Error = Error> + 'a {
    env.docker
        .create_image(Some(CreateImageOptions {
            from_image: image_name,
            tag: "latest",
            ..Default::default()
        }))
        .then(|result| {
            println!("{:?}", result);
            result
        })
        .collect()
}

// Bollard doesn't support image build yet :(
// We are building images by executing docker client manually
pub fn build(image_name: &str, build_path: impl AsRef<Path>) -> SomaResult<()> {
    Command::new("docker")
        .args(&[
            "build",
            "--pull",
            "--force-rm",
            "-t",
            image_name,
            &build_path.as_ref().to_string_lossy(),
        ])
        .status()?;
    Ok(())
}

pub fn create<'a>(
    env: &'a Environment<impl Connect + 'static, impl Printer>,
    image_name: &'a str,
) -> impl Future<Item = String, Error = Error> + 'a {
    let mut labels = HashMap::new();
    labels.insert(LABEL_KEY_VERSION, VERSION);
    labels.insert(LABEL_KEY_USERNAME, &env.username());
    labels.insert(LABEL_KEY_REPOSITORY, "test");
    env.docker
        .create_container(
            None::<CreateContainerOptions<String>>,
            Config {
                image: Some(image_name),
                labels: Some(labels),
                ..Default::default()
            },
        )
        .map(|container_results| container_results.id)
}

pub fn start(
    env: &Environment<impl Connect + 'static, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
}
