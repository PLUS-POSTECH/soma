use bollard::Docker;
use clap::{App, Arg, crate_version, SubCommand};
use crossterm::{cursor, terminal, Terminal, TerminalCursor};
use crossterm::terminal::ClearType;
use hyper::client::connect::Connect;
use tokio::runtime::current_thread::Runtime;
use failure::Error;

use soma::Environment;
use soma::Printer;

struct TerminalPrinter {
    cursor: TerminalCursor<'static>,
    terminal: Terminal<'static>,
}

impl TerminalPrinter {
    fn new() -> TerminalPrinter {
        TerminalPrinter {
            cursor: cursor(),
            terminal: terminal(),
        }
    }
}

impl Printer for TerminalPrinter {
    type Handle = (u16, u16);

    fn get_current_handle(&self) -> Self::Handle {
        let handle = self.cursor.pos();
        println!();
        handle
    }

    fn write_line_at(&mut self, handle: &Self::Handle, message: &str) {
        self.cursor.save_position();
        self.cursor.goto(handle.0, handle.1);
        self.terminal.clear(ClearType::CurrentLine);
        println!("{}", message);
        self.cursor.reset_position();
    }

    fn write_line(&mut self, message: &str) {
        println!("{}", message);
    }
}

#[cfg(windows)]
fn connect_default() -> Result<Docker<impl Connect>, Error> {
    Docker::connect_with_named_pipe_defaults()
}

#[cfg(unix)]
fn connect_default() -> Result<Docker<impl Connect>, Error> {
    Docker::connect_with_unix_defaults()
}

fn default_setup() -> (Environment<impl Connect>, TerminalPrinter) {
    (
        Environment::new(
            connect_default().expect("failed to connect to docker"),
            Runtime::new().expect("failed to create runtime"),
        ),
        TerminalPrinter::new(),
    )
}


fn main() {
    let matches = App::new("soma")
        .version(crate_version!())
        .about("Your one-stop CTF problem management tool")
        .subcommand(SubCommand::with_name("list")
            .about("shows the list of containers"))
        .subcommand(SubCommand::with_name("run-hello")
            .about("runs docker hello-world container"))
        .get_matches();

    match matches.subcommand() {
        ("list", _) => {
            let (mut env, mut printer) = default_setup();
            match env.list() {
                Ok(images) => {
                    let message = images.iter().map(|container| {
                        format!("{:?}", container)
                    }).collect::<Vec<_>>().join("\n");
                    printer.write_line(&message);
                }
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            }
        }
        ("run-hello", _) => {
            let (mut env, mut printer) = default_setup();
            env.pull(&mut printer, "hello-world");

            let create_result = env.create("hello-world");
            println!("Created a container: {:?}", create_result);
            if let Ok(container_name) = create_result {
                println!("Started a container: {:?}", env.start(&container_name));
            }
        }
        _ => {
            println!("I don't understand...");
        }
    }
}
