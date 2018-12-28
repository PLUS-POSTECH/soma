use bollard::Docker;
use clap::{crate_version, App, AppSettings};
use failure::Error;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;

use soma::Printer;
use soma::Soma;

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

fn default_setup() -> (Soma<impl Connect>, impl Printer) {
    (
        Soma::new(
            connect_default().expect("failed to connect to docker"),
            Runtime::new().expect("failed to create tokio runtime"),
        ),
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

    let (soma, printer) = default_setup();

    match matches.subcommand() {
        (AddCommand::NAME, Some(matches)) => add_command.handle_match(matches, soma, printer),
        (ListCommand::NAME, Some(matches)) => list_command.handle_match(matches, soma, printer),
        _ => unreachable!(),
    }
}
