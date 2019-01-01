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

#[derive(Clone, Copy, Debug)]
pub enum ImageStatus {
    Normal,
    VersionMismatch,
    NoVersionFound,
}

#[derive(Debug)]
pub struct SomaImage {
    repository_name: String,
    image: APIImages,
    status: ImageStatus,
}

impl SomaImage {
    pub fn new(repository_name: String, image: APIImages, status: ImageStatus) -> SomaImage {
        SomaImage {
            repository_name,
            image,
            status,
        }
    }

    pub fn repository_name(&self) -> &String {
        &self.repository_name
    }

    pub fn image(&self) -> &APIImages {
        &self.image
    }

    pub fn status(&self) -> ImageStatus {
        self.status
    }
}

pub fn list(
    env: &Environment<impl Connect + 'static, impl Printer>,
) -> impl Future<Item = Vec<SomaImage>, Error = Error> {
    let username = env.username().clone();
    env.docker
        .list_images(Some(ListImagesOptions::<String> {
            ..Default::default()
        }))
        .map(move |images| -> Vec<SomaImage> {
            images
                .into_iter()
                .filter(|image| match &image.labels {
                    Some(labels) => match labels.get(LABEL_KEY_USERNAME) {
                        Some(image_username) => image_username == &username,
                        None => false,
                    },
                    None => false,
                })
                .map(|image| {
                    let repository_name =
                        match image.labels.as_ref().unwrap().get(LABEL_KEY_REPOSITORY) {
                            Some(name) => name.clone(),
                            None => "**NONAME**".to_string(),
                        };
                    let status = match image.labels.as_ref().unwrap().get(LABEL_KEY_VERSION) {
                        Some(image_version) => match image_version.as_str() {
                            VERSION => ImageStatus::Normal,
                            _ => ImageStatus::VersionMismatch,
                        },
                        None => ImageStatus::NoVersionFound,
                    };
                    SomaImage::new(repository_name, image, status)
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
