use std::collections::HashMap;

use bollard::container::{
    APIContainers, Config, CreateContainerOptions, HostConfig, ListContainersOptions, PortBinding,
    PruneContainersOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::{
    APIImages, BuildImageOptions, ListImagesOptions, PruneImagesOptions, RemoveImageOptions,
};
use bollard::Docker;
use failure::Error;
use futures::{Future, Stream};
use hyper::client::connect::Connect;

use crate::prelude::*;
use crate::problem::Problem;
use crate::{Environment, Printer, VERSION};

const LABEL_KEY_VERSION: &str = "soma.version";
const LABEL_KEY_USERNAME: &str = "soma.username";
const LABEL_KEY_REPOSITORY: &str = "soma.repository";
const LABEL_KEY_PROBLEM: &str = "soma.problem";

#[cfg(windows)]
pub fn connect_default() -> SomaResult<Docker<impl Connect>> {
    Docker::connect_with_named_pipe("npipe:////./pipe/docker_engine", 600)
}

#[cfg(unix)]
pub fn connect_default() -> SomaResult<Docker<impl Connect>> {
    Docker::connect_with_unix("unix:///var/run/docker.sock", 600)
}

#[derive(Clone, Copy, Debug)]
pub enum VersionStatus {
    Normal,
    VersionMismatch,
    NoVersionFound,
}

#[derive(Debug)]
pub struct SomaImage {
    repo_name: String,
    prob_name: String,
    image: APIImages,
    status: VersionStatus,
}

impl SomaImage {
    pub fn new(
        repo_name: String,
        prob_name: String,
        image: APIImages,
        status: VersionStatus,
    ) -> SomaImage {
        SomaImage {
            repo_name,
            prob_name,
            image,
            status,
        }
    }

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn prob_name(&self) -> &String {
        &self.prob_name
    }

    pub fn image(&self) -> &APIImages {
        &self.image
    }

    pub fn status(&self) -> VersionStatus {
        self.status
    }
}

type SomaFilter = HashMap<String, Vec<String>>;

struct SomaFilterBuilder {
    label_filter: Vec<String>,
}

impl SomaFilterBuilder {
    fn new() -> SomaFilterBuilder {
        SomaFilterBuilder {
            label_filter: vec![],
        }
    }

    fn append_filter(mut self, key: String, value: String) -> SomaFilterBuilder {
        self.label_filter.push(format!("{}={}", key, value));
        self
    }

    pub fn append_user(self, username: &str) -> SomaFilterBuilder {
        self.append_filter(LABEL_KEY_USERNAME.to_owned(), username.to_owned())
    }

    pub fn append_prob(self, problem: &Problem) -> SomaFilterBuilder {
        self.append_filter(
            LABEL_KEY_REPOSITORY.to_owned(),
            problem.repo_name().to_owned(),
        )
        .append_filter(LABEL_KEY_PROBLEM.to_owned(), problem.prob_name().to_owned())
    }

    pub fn build(self) -> SomaFilter {
        let mut filter = SomaFilter::new();
        filter.insert("label".to_owned(), self.label_filter);
        filter
    }
}

pub fn image_exists(images: &[SomaImage], image_name: &str) -> bool {
    images.iter().any(|image| match &image.image().repo_tags {
        Some(tags) => tags
            .iter()
            .any(|tag| tag.starts_with(format!("{}:", image_name).as_str())),
        None => false,
    })
}

pub fn image_from_repo_exists(images: &[SomaImage], repo_name: &str) -> bool {
    images.iter().any(|image| image.repo_name() == repo_name)
}

pub fn image_from_prob_exists(images: &[SomaImage], problem: &Problem) -> bool {
    images.iter().any(|image| {
        image.repo_name() == problem.repo_name() && image.prob_name() == problem.prob_name()
    })
}

#[derive(Debug)]
pub struct SomaContainer {
    repo_name: String,
    prob_name: String,
    container: APIContainers,
    status: VersionStatus,
}

impl SomaContainer {
    pub fn new(
        repo_name: String,
        prob_name: String,
        container: APIContainers,
        status: VersionStatus,
    ) -> SomaContainer {
        SomaContainer {
            repo_name,
            prob_name,
            container,
            status,
        }
    }

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn prob_name(&self) -> &String {
        &self.prob_name
    }

    pub fn container(&self) -> &APIContainers {
        &self.container
    }

    pub fn status(&self) -> VersionStatus {
        self.status
    }
}

pub fn container_exists(containers: &[SomaContainer], container_id: &str) -> bool {
    containers
        .iter()
        .any(|container| container.container().id == container_id)
}

pub fn container_from_prob_exists(containers: &[SomaContainer], problem: &Problem) -> bool {
    containers.iter().any(|container| {
        container.repo_name() == problem.repo_name() && container.prob_name() == problem.prob_name()
    })
}

pub fn container_from_prob_running(containers: &[SomaContainer], problem: &Problem) -> bool {
    containers.iter().any(|container| {
        container.repo_name() == problem.repo_name()
            && container.prob_name() == problem.prob_name()
            && container.container().state == "running"
    })
}

pub fn containers_from_prob(
    containers: Vec<SomaContainer>,
    problem: &Problem,
) -> Vec<SomaContainer> {
    containers
        .into_iter()
        .filter(|container| {
            container.repo_name() == problem.repo_name()
                && container.prob_name() == problem.prob_name()
        })
        .collect()
}

pub fn list_containers(
    env: &Environment<impl Connect, impl Printer>,
) -> impl Future<Item = Vec<SomaContainer>, Error = Error> {
    let soma_filter = SomaFilterBuilder::new().append_user(env.username()).build();
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
                    if let (Some(repo_name), Some(prob_name)) = (
                        labels.get(LABEL_KEY_REPOSITORY),
                        labels.get(LABEL_KEY_PROBLEM),
                    ) {
                        let status = match labels.get(LABEL_KEY_VERSION) {
                            Some(version) if version == VERSION => VersionStatus::Normal,
                            Some(_) => VersionStatus::VersionMismatch,
                            None => VersionStatus::NoVersionFound,
                        };
                        Some(SomaContainer::new(
                            repo_name.to_owned(),
                            prob_name.to_owned(),
                            container,
                            status,
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        })
}

pub fn list_images(
    env: &Environment<impl Connect, impl Printer>,
) -> impl Future<Item = Vec<SomaImage>, Error = Error> {
    let soma_filter = SomaFilterBuilder::new().append_user(env.username()).build();
    env.docker
        .list_images(Some(ListImagesOptions::<String> {
            filters: soma_filter,
            ..Default::default()
        }))
        .map(move |images| -> Vec<SomaImage> {
            images
                .into_iter()
                .filter_map(|image| {
                    // soma_filter guarantees label existence
                    let labels = image.labels.as_ref().unwrap();
                    if let (Some(repo_name), Some(prob_name)) = (
                        labels.get(LABEL_KEY_REPOSITORY),
                        labels.get(LABEL_KEY_PROBLEM),
                    ) {
                        let status = match labels.get(LABEL_KEY_VERSION) {
                            Some(version) if version == VERSION => VersionStatus::Normal,
                            Some(_) => VersionStatus::VersionMismatch,
                            None => VersionStatus::NoVersionFound,
                        };
                        Some(SomaImage::new(
                            repo_name.to_owned(),
                            prob_name.to_owned(),
                            image,
                            status,
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        })
}

pub fn build<'a>(
    env: &'a Environment<impl Connect, impl Printer>,
    image_name: &'a str,
    build_context: Vec<u8>,
) -> impl Future<Item = (), Error = Error> + 'a {
    let build_options = BuildImageOptions {
        t: image_name,
        pull: true,
        forcerm: true,
        ..Default::default()
    };

    env.docker
        .build_image(build_options, None, Some(build_context.into()))
        .fold((), move |_, build_image_result| {
            use bollard::image::BuildImageResults::*;
            match build_image_result {
                BuildImageStream { stream } => {
                    let message = stream.trim();
                    if message != "" {
                        env.printer().write_line(message)
                    }
                    Ok(())
                }
                BuildImageError { error, .. } => {
                    env.printer().write_line(error.trim());
                    Err(SomaError::DockerBuildFailed)
                }
                _ => Ok(()),
            }
        })
}

pub fn docker_labels<'a>(
    env: &'a Environment<impl Connect, impl Printer>,
    problem: &'a Problem,
) -> HashMap<&'static str, &'a str> {
    vec![
        (LABEL_KEY_VERSION, VERSION),
        (LABEL_KEY_USERNAME, &env.username()),
        (LABEL_KEY_REPOSITORY, problem.repo_name()),
        (LABEL_KEY_PROBLEM, problem.prob_name()),
    ]
    .into_iter()
    .collect()
}

#[allow(clippy::implicit_hasher)]
pub fn create<'a>(
    env: &'a Environment<impl Connect, impl Printer>,
    labels: HashMap<&'a str, &'a str>,
    image_name: &'a str,
    port_str: &'a str,
) -> impl Future<Item = String, Error = Error> + 'a {
    let mut port_bindings = HashMap::new();
    port_bindings.insert(
        "1337/tcp",
        vec![PortBinding {
            host_ip: "",
            host_port: port_str,
        }],
    );

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        ..Default::default()
    };

    env.docker
        .create_container(
            None::<CreateContainerOptions<String>>,
            Config {
                image: Some(image_name),
                labels: Some(labels),
                host_config: Some(host_config),
                ..Default::default()
            },
        )
        .map(|container_results| container_results.id)
}

pub fn remove_image(
    env: &Environment<impl Connect, impl Printer>,
    image_name: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .remove_image(image_name, None::<RemoveImageOptions>, None)
        .map(|_| ())
}

pub fn remove_container(
    env: &Environment<impl Connect, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .remove_container(container_id, None::<RemoveContainerOptions>)
}

pub fn prune_images_from_prob(
    env: &Environment<impl Connect, impl Printer>,
    problem: &Problem,
) -> impl Future<Item = (), Error = Error> {
    let soma_filter = SomaFilterBuilder::new()
        .append_user(env.username())
        .append_prob(problem)
        .build();
    env.docker
        .prune_images(Some(PruneImagesOptions {
            filters: soma_filter,
        }))
        .map(|_| ())
}

pub fn prune_containers_from_prob(
    env: &Environment<impl Connect, impl Printer>,
    problem: &Problem,
) -> impl Future<Item = (), Error = Error> {
    let soma_filter = SomaFilterBuilder::new()
        .append_user(env.username())
        .append_prob(problem)
        .build();
    env.docker
        .prune_containers(Some(PruneContainersOptions {
            filters: soma_filter,
        }))
        .map(|_| ())
}

pub fn start(
    env: &Environment<impl Connect, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
}

pub fn stop(
    env: &Environment<impl Connect, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    env.docker
        .stop_container(container_id, None::<StopContainerOptions>)
}
