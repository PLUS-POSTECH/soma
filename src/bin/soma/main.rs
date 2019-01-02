use std::string::ToString;

use bollard::Docker;
use clap::{App, AppSettings};
use hyper::client::connect::Connect;
use whoami::username;

use soma::data_dir::DataDirectory;
use soma::error::Result as SomaResult;
use soma::{Environment, Printer, VERSION};

use crate::commands::*;
use crate::terminal_printer::TerminalPrinter;

mod commands;
mod terminal_printer;

#[cfg(windows)]
fn connect_default() -> SomaResult<Docker<impl Connect>> {
    Docker::connect_with_named_pipe_defaults()
}

#[cfg(unix)]
fn connect_default() -> SomaResult<Docker<impl Connect>> {
    Docker::connect_with_unix_defaults()
}

fn cli_env(data_dir: DataDirectory) -> Environment<impl Connect, impl Printer> {
    Environment::new(
        username(),
        data_dir,
        connect_default().expect("failed to connect to docker"),
        TerminalPrinter::new(),
    )
}

fn main_result() -> SomaResult<()> {
    let add_command = AddCommand::new();
    let list_command = ListCommand::new();
    let clone_command = CloneCommand::new();
    let fetch_command = FetchCommand::new();

    let matches = App::new("soma")
        .version(VERSION)
        .about("Your one-stop CTF problem management tool")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(add_command.app())
        .subcommand(list_command.app())
        .subcommand(clone_command.app())
        .subcommand(fetch_command.app())
        .get_matches();

    let data_dir = DataDirectory::new()?;
    let env = cli_env(data_dir);

    match matches.subcommand() {
        (AddCommand::NAME, Some(matches)) => add_command.handle_match(env, matches),
        (ListCommand::NAME, Some(matches)) => list_command.handle_match(env, matches),
        (CloneCommand::NAME, Some(matches)) => clone_command.handle_match(env, matches),
        (FetchCommand::NAME, Some(matches)) => fetch_command.handle_match(env, matches),
        _ => unreachable!(),
    }
}

fn main() {
    if let Err(err) = main_result() {
        eprintln!("{}", err.to_string());
        std::process::exit(1);
    }
}
