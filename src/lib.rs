use std::cell::{RefCell, RefMut};

use bollard::Docker;
use clap::crate_version;
use hyper::client::connect::Connect;

use crate::data_dir::DataDirectory;
use crate::prelude::*;
use crate::repository::RepositoryManager;

pub mod data_dir;
pub mod docker;
pub mod error;
pub mod ops;
pub mod prelude;
pub mod problem;
pub mod repository;
pub mod template;

pub const VERSION: &str = crate_version!();

pub trait Printer {
    type Handle;

    fn get_current_handle(&mut self) -> Self::Handle;
    fn write_line_at(&mut self, handle: &Self::Handle, message: &str);
    fn write_line(&mut self, message: &str);
}

pub struct Environment<'a, C: 'static, P: Printer + 'static> {
    username: String,
    repo_manager: RepositoryManager<'a>,
    docker: Docker<C>,
    printer: RefCell<P>,
}

impl<'a, C, P> Environment<'a, C, P>
where
    C: Connect,
    P: Printer,
{
    pub fn new(
        username: String,
        data_dir: &'a mut DataDirectory,
        docker: Docker<C>,
        printer: P,
    ) -> SomaResult<Environment<'a, C, P>> {
        let repo_manager = data_dir.register::<RepositoryManager>()?;

        Ok(Environment {
            username,
            repo_manager,
            docker,
            printer: RefCell::new(printer),
        })
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn printer(&self) -> RefMut<P> {
        self.printer.borrow_mut()
    }

    pub fn repo_manager(&self) -> &RepositoryManager<'a> {
        &self.repo_manager
    }

    pub fn repo_manager_mut(&mut self) -> &mut RepositoryManager<'a> {
        &mut self.repo_manager
    }
}
