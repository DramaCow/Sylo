use std::fmt::{Error, Write, Arguments};

#[derive(Default)]
pub struct IndentWriter<W: Write> {
    fmt: W,
    indent_count: usize,
    written_to_line: bool,
}

impl<W: Write> IndentWriter<W> {
    #[must_use]
    pub fn new(fmt: W) -> Self {
        Self {
            fmt,
            indent_count: 0,
            written_to_line: false,
        }
    }

    /// # Errors
    pub fn write_fmt(&mut self, fmt: Arguments) -> Result<(), Error> {
        <Self as Write>::write_fmt(self, fmt)
    }

    pub fn indent(&mut self) -> &mut Self {
        self.indent_count += 4;
        self
    }

    pub fn unindent(&mut self) -> &mut Self {
        if self.indent_count > 0 {
            self.indent_count -= 4;
        }
        self
    }

    #[must_use]
    pub fn build(self) -> W {
        self.fmt
    }
}

impl<W: Write> Write for IndentWriter<W> {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        let mut split = s.split('\n');

        if !self.written_to_line {
            self.written_to_line = true;
            write!(self.fmt, "{:width$}", "", width = self.indent_count)?;
        }
        write!(self.fmt, "{}", split.next().unwrap())?;

        for substr in split {
            if substr.is_empty() {
                self.written_to_line = false;
                write!(self.fmt, "\n")?;
            } else {
                self.written_to_line = true;
                write!(self.fmt, "\n{:width$}{}", "", substr, width = self.indent_count)?;
            }
        }

        Ok(())
    }
}