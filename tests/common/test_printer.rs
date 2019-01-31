use soma::Printer;

pub struct TestPrinter {
    output: String,
}

impl TestPrinter {
    pub fn new() -> TestPrinter {
        TestPrinter {
            output: String::new(),
        }
    }
}

impl TestPrinter {
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl Printer for TestPrinter {
    // TestPrinter does not support write_line_at
    type Handle = ();

    fn get_current_handle(&mut self) -> Self::Handle {
        ()
    }

    fn write_line_at(&mut self, _handle: &Self::Handle, message: &str) {
        self.write_line(message)
    }

    fn write_line(&mut self, message: &str) {
        println!("{}", message);
        self.output.push_str(message);
        self.output.push('\n');
    }
}
