use std::cell::{RefCell, RefMut};

use bollard::Docker;
use clap::crate_version;
use hyper::client::connect::Connect;

use crate::data_dir::DataDirectory;

pub mod data_dir;
pub mod docker;
pub mod error;
pub mod repo;
pub mod template;

pub const VERSION: &'static str = crate_version!();

pub trait Printer {
    type Handle;

    fn get_current_handle(&self) -> Self::Handle;
    fn write_line_at(&mut self, handle: &Self::Handle, message: &str);
    fn write_line(&mut self, message: &str);
}

pub struct Environment<C, P: Printer> {
    username: String,
    data_dir: DataDirectory,
    docker: Docker<C>,
    printer: RefCell<P>,
}

impl<C, P: Printer> Environment<C, P>
where
    C: 'static + Connect,
{
    pub fn new(
        username: String,
        data_dir: DataDirectory,
        docker: Docker<C>,
        printer: P,
    ) -> Environment<C, P> {
        Environment {
            username,
            data_dir,
            docker,
            printer: RefCell::new(printer),
        }
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn data_dir(&self) -> &DataDirectory {
        &self.data_dir
    }

    pub fn printer(&self) -> RefMut<P> {
        self.printer.borrow_mut()
    }
}
