use bollard::container::{Config, CreateContainerOptions, StartContainerOptions};
use bollard::image::APIImages;
use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::Docker;
use failure::Error;
use futures::stream::Stream;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;

pub mod error;
pub mod repo;

type DockerResult<T> = Result<T, Error>;

pub trait Printer: Send + Sync {
    type Handle;

    fn get_current_handle(&self) -> Self::Handle;
    fn write_line_at(&mut self, handle: &Self::Handle, message: &str);
    fn write_line(&mut self, message: &str);
}

pub struct Soma<C> {
    docker: Docker<C>,
    runtime: Runtime,
}

impl<C> Soma<C>
where
    C: 'static + Connect,
{
    pub fn new(docker: Docker<C>, runtime: Runtime) -> Soma<C> {
        Soma { docker, runtime }
    }

    pub fn list(&mut self) -> DockerResult<Vec<APIImages>> {
        self.runtime
            .block_on(self.docker.list_images(Some(ListImagesOptions::<String> {
                all: true,
                ..Default::default()
            })))
    }

    pub fn pull(&mut self, _printer: &mut impl Printer, image_name: &str) {
        self.runtime.block_on(
            self.docker
                .create_image(Some(CreateImageOptions {
                    from_image: image_name,
                    tag: "latest",
                    ..Default::default()
                }))
                .then(|result| {
                    println!("{:?}", result);
                    result
                })
                .collect(),
        );
    }

    pub fn create(&mut self, image_name: &str) -> DockerResult<String> {
        self.runtime
            .block_on(self.docker.create_container(
                None::<CreateContainerOptions<String>>,
                Config {
                    image: Some(image_name),
                    ..Default::default()
                },
            ))
            .map(|container_results| container_results.id)
    }

    pub fn start(&mut self, container_id: &str) -> DockerResult<()> {
        self.runtime.block_on(
            self.docker
                .start_container(container_id, None::<StartContainerOptions<String>>),
        )
    }
}
