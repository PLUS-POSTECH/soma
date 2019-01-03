use soma::Printer;

pub struct TestPrinter {
    buffer: String,
}

impl TestPrinter {
    pub fn new() -> TestPrinter {
        TestPrinter {
            buffer: String::new(),
        }
    }
}

impl Printer for TestPrinter {
    // TestPrinter does not support write_line_at
    type Handle = ();

    fn get_current_handle(&self) -> Self::Handle {
        ()
    }

    fn write_line_at(&mut self, _handle: &Self::Handle, message: &str) {
        self.write_line(message)
    }

    fn write_line(&mut self, message: &str) {
        self.buffer.push_str(message);
    }
}
