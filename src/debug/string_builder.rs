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

    #[must_use]
    pub fn get_indent(&self) -> usize {
        self.indent_count
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

    pub fn debug_write_slice<T: std::fmt::Debug>(&mut self, slice: &[T]) {
        self.writeln("[");
        self.indent();
        {
            let tab_size = self.get_indent();
            let slice_fmt = slice.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>();
            let cell_size = slice_fmt.iter().map(String::len).max().unwrap();
            let cells_per_row = (80 - tab_size) / (cell_size + 2);
            for row in slice_fmt.chunks(cells_per_row) {
                for cell in row {
                    self.write(&format!("{:padding$}, ", cell, padding=cell_size));
                }
                self.newline();
            }
        }
        self.unindent();
        self.write("]");
    }

    #[must_use]
    pub fn build(self) -> String {
        self.string
    }
}