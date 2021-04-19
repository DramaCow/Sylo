mod regex;
pub use self::regex::{
    RegEx,
    Operator,
};

mod unicode;
pub use self::unicode::{
    basic_latin,
    basic_multilingual_plane,
    non_compatibility_char,
};

mod dfa;
pub use self::dfa::DFA;

mod table;
pub use self::table::{
    ScanningTable,
    Command,
};

mod scan;
pub use self::scan::{
    Token,
    Scan,
    ScanError
};

mod array_scanning_table;
pub use self::array_scanning_table::ArrayScanningTable;

/// Constructs a `RegEx` that recognizes some input string only.
#[must_use]
pub fn literal(s: &str) -> RegEx {
    s.bytes().fold(RegEx::empty(), |r, byte| {
        r.then(&RegEx::set(CharSet::point(byte)))
    })
}

/// Constructs a `RegEx` that recognizes any char in a string.
#[must_use]
pub fn any(s: &str) -> RegEx {
    s.chars().fold(RegEx::empty(), |r, c| {
        let mut buffer: [u8; 4] = [0; 4];
        r.or(&literal(c.encode_utf8(&mut buffer)))
    })
}

/// Constructs a `RegEx` that recognizes all chars within a provided range (inclusive).
/// Also accounts for char ranges that span different number of bytes.
#[must_use]
pub fn range(from: char, to: char) -> RegEx {
    #[allow(clippy::cast_possible_truncation)]
    fn range32(from: u32, to: u32, optional: bool) -> RegEx {
        fn byte_range(from: u8, to: u8) -> RegEx {
            RegEx::set(CharSet::range(from, to))
        }
    
        let (a_low, a_high) = (from as u8 + optional as u8, from >> 8);
        let (b_low, b_high) = (to as u8, to >> 8);
    
        let regex = {
            if b_high == 0 {
                byte_range(a_low, b_low)
            } else if a_high == b_high {
                byte_range(a_low, b_low).then(&range32(a_high, b_high, false))
            } else if a_low == u8::MIN && b_low == u8::MAX {
                byte_range(u8::MIN, u8::MAX).then(&range32(a_high, b_high, a_high == 0))
            } else if a_low == u8::MIN {
                byte_range(u8::MIN, b_low).then(&range32(a_high, b_high, a_high == 0))
                .or(&byte_range(b_low + 1, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            } else if b_low == u8::MAX {
                byte_range(u8::MIN, a_low - 1).then(&range32(a_high + 1, b_high, false))
                .or(&byte_range(a_low, u8::MAX).then(&range32(a_high, b_high, a_high == 0)))
            } else if b_low >= a_low {
                byte_range(u8::MIN, a_low - 1).then(&range32(a_high + 1, b_high, false))
                .or(&byte_range(a_low, b_low).then(&range32(a_high, b_high, a_high == 0)))
                .or(&byte_range(b_low + 1, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            } else if b_high > a_high + 1 && b_low + 1 < a_low {
                byte_range(u8::MIN, b_low).then(&range32(a_high + 1, b_high, false))
                .or(&byte_range(b_low + 1, a_low - 1).then(&range32(a_high + 1, b_high - 1, false)))
                .or(&byte_range(a_low, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            } else {
                byte_range(u8::MIN, b_low).then(&range32(a_high + 1, b_high, false))
                .or(&byte_range(a_low, u8::MAX).then(&range32(a_high, b_high - 1, a_high == 0)))
            }
        };
    
        if optional { regex.opt() } else { regex }
    }
    
    range32(from as u32, to as u32, false)
}

// =================
// === INTERNALS ===
// =================

mod char_set;
use self::char_set::CharSet;

#[cfg(test)]
mod tests;