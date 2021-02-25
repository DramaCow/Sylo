#[derive(Default)]
pub struct StringBuilder {
    string: String,
    indent_count: usize,
    indent: String,
    written_to_line: bool,
}

impl StringBuilder {
    #[must_use]
    pub fn new () -> Self {
        Self {
            string: String::new(),
            indent_count: 0,
            indent: String::new(),
            written_to_line: false,
        }
    }

    pub fn write(&mut self, text: &str) {
        if !self.written_to_line {
            self.written_to_line = true;
            self.string.push_str(&self.indent);
        }
        self.string.push_str(text);
    }

    pub fn newline(&mut self) {
        self.string.push('\n');
        self.written_to_line = false;
    }

    pub fn writeln(&mut self, text: &str) {
        self.write(text);
        self.newline();
    }

    pub fn indent(&mut self) {
        self.indent_count += 4;
        self.indent = " ".repeat(self.indent_count);
    }

    pub fn unindent(&mut self) {
        if self.indent_count > 0 {
            self.indent_count -= 4;
            self.indent = " ".repeat(self.indent_count);
        }
    }

    #[must_use]
    pub fn build(self) -> String {
        self.string
    }
}