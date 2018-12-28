use bollard::Docker;
use clap::{crate_version, App, AppSettings};
use failure::Error;
use hyper::client::connect::Connect;

use soma::Config;
use soma::Printer;

use crate::commands::{add::AddCommand, list::ListCommand, SomaCommand};
use crate::terminal_printer::TerminalPrinter;

mod commands;
mod terminal_printer;

#[cfg(windows)]
fn connect_default() -> Result<Docker<impl Connect>, Error> {
    Docker::connect_with_named_pipe_defaults()
}

#[cfg(unix)]
fn connect_default() -> Result<Docker<impl Connect>, Error> {
    Docker::connect_with_unix_defaults()
}

fn default_setup() -> Config<impl Connect, impl Printer> {
    Config::new(
        connect_default().expect("failed to connect to docker"),
        TerminalPrinter::new(),
    )
}

fn main() {
    let add_command: AddCommand = AddCommand::new();
    let list_command: ListCommand = ListCommand::new();

    let matches = App::new("soma")
        .version(crate_version!())
        .about("Your one-stop CTF problem management tool")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(add_command.app())
        .subcommand(list_command.app())
        .get_matches();

    let config = default_setup();

    match matches.subcommand() {
        (AddCommand::NAME, Some(matches)) => add_command.handle_match(config, matches),
        (ListCommand::NAME, Some(matches)) => list_command.handle_match(config, matches),
        _ => unreachable!(),
    }
}
