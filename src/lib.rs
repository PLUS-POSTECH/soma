use bollard::Docker;
use bollard::image::APIImages;
use bollard::image::ListImagesOptions;
use failure::Error;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;

type DockerResult<T> = Result<T, Error>;

pub trait Printer: Send + Sync {
    type Handle;

    fn get_current_handle(&self) -> Self::Handle;
    fn write_line_at(&mut self, handle: &Self::Handle, message: &str);
    fn write_line(&mut self, message: &str);
}

pub struct Environment<C> {
    docker: Docker<C>,
    runtime: Runtime,
}

impl<C> Environment<C>
    where C: 'static + Connect + Sync
{
    pub fn new(docker: Docker<C>, runtime: Runtime) -> Environment<C> {
        Environment {
            docker,
            runtime,
        }
    }

    pub fn list(&mut self) -> DockerResult<Vec<APIImages>> {
        self.runtime.block_on(
            self.docker.list_images(Some(ListImagesOptions::<String> {
                all: true,
                ..Default::default()
            })))
    }

    pub fn pull(&mut self, printer: &mut impl Printer, image_name: &str) {}
}

/*
pub fn create_hello() -> impl Future<Item=Vec<(serde_json::value::Value, ContainerCreateInfo)>, Error=shiplift::Error> {
    let image_name = "hello-world";
    let container_options = ContainerOptions::builder(image_name).build();

    DOCKER.images().pull(&PullOptions::builder().image(image_name).tag("latest").build())
        .and_then(move |pull_result| {
            DOCKER.containers().create(&container_options).map(
                |create_result| (pull_result, create_result)
            )
        }).collect()
}
*/
