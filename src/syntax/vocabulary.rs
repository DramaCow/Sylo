/// Information on how to display lexical symbols.
#[derive(Clone)]
pub struct Vocabulary {
    pub symbolic_names: Vec<String>,
}

impl Vocabulary {
    #[must_use]
    pub fn new(symbolic_names: Vec<String>) -> Self {
        Self {
            symbolic_names,
        }
    }

    #[must_use]
    pub fn get_display_name(&self, index: usize) -> &str {
        &self.symbolic_names[index]
    }
}

