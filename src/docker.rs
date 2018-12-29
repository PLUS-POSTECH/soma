use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, StartContainerOptions,
};
use bollard::image::{APIImages, CreateImageOptions, CreateImageResults, ListImagesOptions};
use failure::Error;
use futures::stream::Stream;
use futures::Future;
use hyper::client::connect::Connect;

use crate::Config;
use crate::Printer;

pub fn list(
    config: &Config<impl Connect + 'static, impl Printer>,
) -> impl Future<Item = Vec<APIImages>, Error = Error> {
    config.docker.list_images(Some(ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    }))
}

pub fn pull<'a>(
    config: &Config<impl Connect + 'static, impl Printer>,
    image_name: &'a str,
) -> impl Future<Item = Vec<CreateImageResults>, Error = Error> + 'a {
    config
        .docker
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

pub fn create<'a>(
    config: &Config<impl Connect + 'static, impl Printer>,
    image_name: &'a str,
) -> impl Future<Item = String, Error = Error> + 'a {
    config
        .docker
        .create_container(
            None::<CreateContainerOptions<String>>,
            ContainerConfig {
                image: Some(image_name),
                ..Default::default()
            },
        )
        .map(|container_results| container_results.id)
}

pub fn start(
    config: &Config<impl Connect + 'static, impl Printer>,
    container_id: &str,
) -> impl Future<Item = (), Error = Error> {
    config
        .docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
}
