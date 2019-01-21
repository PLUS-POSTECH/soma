use crossterm::terminal::ClearType;
use crossterm::{cursor, terminal, Terminal, TerminalCursor};

use soma::prelude::*;
use soma::Printer;

pub struct TerminalPrinter {
    cursor: TerminalCursor<'static>,
    terminal: Terminal<'static>,
}

impl TerminalPrinter {
    pub fn new() -> TerminalPrinter {
        TerminalPrinter {
            cursor: cursor(),
            terminal: terminal(),
        }
    }
}

impl Printer for TerminalPrinter {
    type Handle = (u16, u16);

    fn get_current_handle(&mut self) -> Self::Handle {
        let handle = self.cursor.pos();
        self.write_line("");
        handle
    }

    fn write_line_at(&mut self, handle: &Self::Handle, message: &str) {
        match || -> SomaResult<()> {
            self.cursor.save_position()?;
            self.cursor.goto(handle.0, handle.1)?;
            self.terminal.clear(ClearType::CurrentLine)?;
            self.write_line(message);
            self.cursor.reset_position()?;
            Ok(())
        }() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error on TerminalPrinter: {}", e);
            }
        }
    }

    fn write_line(&mut self, message: &str) {
        match || -> SomaResult<()> {
            self.terminal.write(message)?;
            self.terminal.write("\n")?;
            Ok(())
        }() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error on TerminalPrinter: {}", e);
            }
        }
    }
}
