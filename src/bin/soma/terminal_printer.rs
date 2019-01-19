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

    fn get_current_handle(&self) -> Self::Handle {
        let handle = self.cursor.pos();
        println!();
        handle
    }

    fn write_line_at(&mut self, handle: &Self::Handle, message: &str) -> SomaResult<()> {
        self.cursor.save_position()?;
        self.cursor.goto(handle.0, handle.1)?;
        self.terminal.clear(ClearType::CurrentLine)?;
        self.terminal.write(message)?;
        self.terminal.write("\n")?;
        self.cursor.reset_position()?;
        Ok(())
    }

    fn write_line(&mut self, message: &str) -> SomaResult<()> {
        self.terminal.write(message)?;
        self.terminal.write("\n")?;
        Ok(())
    }
}
