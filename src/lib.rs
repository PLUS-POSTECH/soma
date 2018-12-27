use std::str::FromStr;

use futures::future::Future;
use http::uri::Uri;
use lazy_static::lazy_static;
use shiplift::builder::ContainerListOptions;
use shiplift::Containers;
use shiplift::Docker;
use shiplift::rep::Container;

lazy_static! {
    static ref DOCKER: Docker = if cfg!(windows) {
            Docker::host(Uri::from_str("tcp://127.0.0.1:2375").unwrap())
        } else {
            Docker::new()
        };
}

pub fn list() -> impl Future<Item = Vec<Container>, Error = shiplift::Error> {
    let container = Containers::new(&DOCKER);
    container.list(&ContainerListOptions::builder().all().build())
}
