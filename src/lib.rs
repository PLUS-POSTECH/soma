use std::cell::{RefCell, RefMut};

use bollard::Docker;
use hyper::client::connect::Connect;
use serde_derive::{Deserialize, Serialize};

use crate::data_dir::DataDirectory;

pub mod data_dir;
pub mod docker;
pub mod error;
pub mod repo;

pub trait Printer {
    type Handle;

    fn get_current_handle(&self) -> Self::Handle;
    fn write_line_at(&mut self, handle: &Self::Handle, message: &str);
    fn write_line(&mut self, message: &str);
}

#[derive(Clone, Deserialize, Serialize)]
pub struct SomaInfo {
    pub username: String,
    pub version: String,
}

pub struct Environment<C, P: Printer> {
    soma_info: SomaInfo,
    data_dir: DataDirectory,
    docker: Docker<C>,
    printer: RefCell<P>,
}

impl<C, P: Printer> Environment<C, P>
where
    C: 'static + Connect,
{
    pub fn new(
        soma_info: SomaInfo,
        data_dir: DataDirectory,
        docker: Docker<C>,
        printer: P,
    ) -> Environment<C, P> {
        Environment {
            soma_info,
            data_dir,
            docker,
            printer: RefCell::new(printer),
        }
    }

    pub fn soma_info(&self) -> &SomaInfo {
        &self.soma_info
    }

    pub fn data_dir(&self) -> &DataDirectory {
        &self.data_dir
    }

    pub fn printer(&self) -> RefMut<P> {
        self.printer.borrow_mut()
    }
}
