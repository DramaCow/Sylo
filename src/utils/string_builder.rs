use std::fmt::{Error, Write, Arguments};

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

    /// # Errors
    pub fn write_fmt(&mut self, fmt: Arguments) -> Result<(), Error> {
        <Self as Write>::write_fmt(self, fmt)
    }

    pub fn newline(&mut self) -> &mut Self {
        self.string.push('\n');
        self.written_to_line = false;
        self
    }

    pub fn indent(&mut self) -> &mut Self {
        self.indent_count += 4;
        self.indent = " ".repeat(self.indent_count);
        self
    }

    pub fn unindent(&mut self) -> &mut Self {
        if self.indent_count > 0 {
            self.indent_count -= 4;
            self.indent = " ".repeat(self.indent_count);
        }
        self
    }

    #[must_use]
    pub fn build(self) -> String {
        self.string
    }
}

impl Write for StringBuilder {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        if !self.written_to_line {
            self.written_to_line = true;
            self.string.push_str(&self.indent);
        }

        self.string.push_str(s);

        if let Some('\n') = s.chars().last() {
            self.written_to_line = false;
        }

        Ok(())
    }
}