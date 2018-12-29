use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::image::{APIImages, CreateImageOptions, CreateImageResults, ListImagesOptions};
use failure::Error;
use futures::stream::Stream;
use futures::Future;
use hyper::client::connect::Connect;

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

pub fn create<'a>(
    env: &Environment<impl Connect + 'static, impl Printer>,
    image_name: &'a str,
) -> impl Future<Item = String, Error = Error> + 'a {
    env.docker
        .create_container(
            None::<CreateContainerOptions<String>>,
            Config {
                image: Some(image_name),
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
