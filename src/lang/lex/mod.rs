pub use self::compile::LexDef;
pub use self::scan::{Token, Scan, ScanError};
use super::Command;

pub struct LexAnalyzer {
    next:     Vec<usize>,
    classes:  Vec<Option<usize>>,
    commands: Vec<Command>,
}

// =================
// === INTERNALS ===
// =================

mod compile;
mod scan;

#[cfg(test)]
mod tests;