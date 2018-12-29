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
use crate::Environment;
use crate::Printer;

pub fn list(
    env: &Environment<impl Connect + 'static, impl Printer>,
) -> impl Future<Item = Vec<APIImages>, Error = Error> {
    env.docker.list_images(Some(ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    }))
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
pub fn build<'a>(image_name: &'a str, build_path: impl AsRef<Path>) -> SomaResult<()> {
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
    labels.insert("soma.version", env.soma_info().version.as_str());
    labels.insert("soma.username", env.soma_info().username.as_str());
    labels.insert("soma.repository", "test");
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
