use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use bollard::container::{
    APIContainers, Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use bollard::image::{
    APIImages, CreateImageOptions, CreateImageResults, ListImagesOptions, RemoveImageOptions,
};
use bollard::Docker;
use failure::Error;
use futures::stream::Stream;
use futures::Future;
use hyper::client::connect::Connect;

use crate::error::{Error as SomaError, Result as SomaResult};
use crate::{Environment, Printer, VERSION};

pub const LABEL_KEY_VERSION: &'static str = "soma.version";
pub const LABEL_KEY_USERNAME: &'static str = "soma.username";
pub const LABEL_KEY_REPOSITORY: &'static str = "soma.repository";

#[cfg(windows)]
pub fn connect_default() -> SomaResult<Docker<impl Connect>> {
    Docker::connect_with_named_pipe_defaults()
}

#[cfg(unix)]
pub fn connect_default() -> SomaResult<Docker<impl Connect>> {
    Docker::connect_with_unix_defaults()
}

#[derive(Clone, Copy, Debug)]
pub enum VersionStatus {
    Normal,
    VersionMismatch,
    NoVersionFound,
}

#[derive(Debug)]
pub struct SomaImage {
    repository_name: String,
    image: APIImages,
    status: VersionStatus,
}

impl SomaImage {
    pub fn new(repository_name: String, image: APIImages, status: VersionStatus) -> SomaImage {
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

    pub fn status(&self) -> VersionStatus {
        self.status
    }
}

#[derive(Debug)]
pub struct SomaContainer {
    repository_name: String,
    container: APIContainers,
    status: VersionStatus,
}

impl SomaContainer {
    pub fn new(
        repository_name: String,
        container: APIContainers,
        status: VersionStatus,
    ) -> SomaContainer {
        SomaContainer {
            repository_name,
            container,
            status,
        }
    }

    pub fn repository_name(&self) -> &String {
        &self.repository_name
    }

    pub fn container(&self) -> &APIContainers {
        &self.container
    }

    pub fn status(&self) -> VersionStatus {
        self.status
    }
}

pub fn list_containers(
    env: &Environment<impl Connect + 'static, impl Printer>,
) -> impl Future<Item = Vec<SomaContainer>, Error = Error> {
    let username = env.username().clone();
    let mut soma_filter = HashMap::new();
    soma_filter.insert(
        "label".to_string(),
        vec![format!(
            "\"{}\"=\"{}\"",
            LABEL_KEY_USERNAME.to_string(),
            username.clone()
        )],
    );
    env.docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            filters: soma_filter,
            ..Default::default()
        }))
        .map(move |containers| -> Vec<SomaContainer> {
            containers
                .into_iter()
                .filter_map(|container| {
                    let labels = &container.labels;
                    let repository_name = match labels.get(LABEL_KEY_REPOSITORY) {
                        Some(name) => name.clone(),
                        None => "**NONAME**".to_string(),
                    };
                    let status = match labels.get(LABEL_KEY_VERSION) {
                        Some(container_version) => match container_version.as_str() {
                            VERSION => VersionStatus::Normal,
                            _ => VersionStatus::VersionMismatch,
                        },
                        None => VersionStatus::NoVersionFound,
                    };
                    Some(SomaContainer::new(repository_name, container, status))
                })
                .collect()
        })
}

pub fn list_images(
    env: &Environment<impl Connect + 'static, impl Printer>,
) -> impl Future<Item = Vec<SomaImage>, Error = Error> {
    let username = env.username().clone();
    let mut soma_filter = HashMap::new();
    soma_filter.insert(
        "label".to_string(),
        format!(
            "\"{}\"=\"{}\"",
            LABEL_KEY_USERNAME.to_string(),
            username.clone()
        ),
    );
    env.docker
        .list_images(Some(ListImagesOptions::<String> {
            filters: soma_filter,
            ..Default::default()
        }))
        .map(move |images| -> Vec<SomaImage> {
            images
                .into_iter()
                .filter_map(|image| {
                    // Unwrap guaranteed by filter option in ListImagesOptions
                    let labels = image.labels.as_ref().unwrap();
                    let repository_name = match labels.get(LABEL_KEY_REPOSITORY) {
                        Some(name) => name.clone(),
                        None => "**NONAME**".to_string(),
                    };
                    let status = match labels.get(LABEL_KEY_VERSION) {
                        Some(image_version) => match image_version.as_str() {
                            VERSION => VersionStatus::Normal,
                            _ => VersionStatus::VersionMismatch,
                        },
                        None => VersionStatus::NoVersionFound,
                    };
                    Some(SomaImage::new(repository_name, image, status))
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
    let status = Command::new("docker")
        .args(&[
            "build",
            "--pull",
            "--force-rm",
            "-t",
            image_name,
            &build_path.as_ref().to_string_lossy(),
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(SomaError::DockerBuildFailError.into())
    }
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

pub fn remove_image(
    env: &Environment<impl Connect + 'static, impl Printer>,
    image_name: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .remove_image(image_name, None::<RemoveImageOptions>)
        .map(|_| ())
}

pub fn start(
    env: &Environment<impl Connect + 'static, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
}

pub fn stop(
    env: &Environment<impl Connect + 'static, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .stop_container(container_id, None::<StopContainerOptions>)
}

pub fn remove_container(
    env: &Environment<impl Connect + 'static, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .remove_container(container_id, None::<RemoveContainerOptions>)
}
