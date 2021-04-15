mod table;
pub use self::table::ScanningTable;

mod scan;
pub use self::scan::{
    Token,
    Scan,
    ScanError
};

mod array_scanning_table;
pub use self::array_scanning_table::ArrayScanningTable;

#[derive(Clone, Copy)]
pub enum Command {
    Skip,
    Emit,
}

// =================
// === INTERNALS ===
// =================

#[cfg(test)]
mod tests;