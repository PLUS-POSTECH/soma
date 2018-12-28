use std::cell::{RefCell, RefMut};

use bollard::Docker;
use hyper::client::connect::Connect;

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

pub struct Config<C, P: Printer> {
    docker: Docker<C>,
    printer: RefCell<P>,
}

impl<C, P: Printer> Config<C, P>
where
    C: 'static + Connect,
{
    pub fn new(docker: Docker<C>, printer: P) -> Config<C, P> {
        Config {
            docker,
            printer: RefCell::new(printer),
        }
    }

    pub fn get_printer(&self) -> RefMut<P> {
        self.printer.borrow_mut()
    }
}
