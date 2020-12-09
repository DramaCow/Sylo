use crate::re::{self, RegEx};
use std::char::from_u32;

#[must_use]
pub fn basic_latin() -> RegEx {
    re::literal("\u{9}")
    .or(&re::literal("\u{A}"))
    .or(&re::literal("\u{D}"))
    .or(&re::range('\u{20}', '\u{7E}'))
}

#[must_use]
pub fn basic_multilingual_plane() -> RegEx {
    re::literal("\u{9}")
    .or(&re::literal("\u{A}"))
    .or(&re::literal("\u{D}"))
    .or(&re::range('\u{20}', '\u{7E}'))
    .or(&re::literal("\u{85}"))
    .or(&re::range('\u{A0}', '\u{D7FF}'))
    .or(&re::range('\u{E000}', '\u{FDCF}'))
    .or(&re::range('\u{FDF0}', '\u{FFFD}'))
}

#[must_use]
pub fn non_compatibility_char() -> RegEx {
    re::literal("\u{9}")
    .or(&re::literal("\u{A}"))
    .or(&re::literal("\u{D}"))
    .or(&re::range('\u{20}', '\u{7E}'))
    .or(&re::literal("\u{85}"))
    .or(&re::range('\u{A0}', '\u{D7FF}'))
    .or(&re::range('\u{E000}', '\u{FDCF}'))
    .or(&re::range('\u{FDF0}', '\u{FFFD}'))
    .or(&re::range('\u{01_0000}', '\u{01_FFFD}'))
    .or(&re::range('\u{02_0000}', '\u{02_FFFD}'))
    .or(&re::range('\u{03_0000}', '\u{03_FFFD}'))
    .or(&re::range('\u{04_0000}', '\u{04_FFFD}'))
    .or(&re::range('\u{05_0000}', '\u{05_FFFD}'))
    .or(&re::range('\u{06_0000}', '\u{06_FFFD}'))
    .or(&re::range('\u{07_0000}', '\u{07_FFFD}'))
    .or(&re::range('\u{08_0000}', '\u{08_FFFD}'))
    .or(&re::range('\u{09_0000}', '\u{09_FFFD}'))
    .or(&re::range('\u{0A_0000}', '\u{0A_FFFD}'))
    .or(&re::range('\u{0B_0000}', '\u{0B_FFFD}'))
    .or(&re::range('\u{0C_0000}', '\u{0C_FFFD}'))
    .or(&re::range('\u{0D_0000}', '\u{0D_FFFD}'))
    .or(&re::range('\u{0E_0000}', '\u{0E_FFFD}'))
    .or(&re::range('\u{0F_0000}', '\u{0F_FFFD}'))
    .or(&re::range('\u{10_0000}', '\u{10_FFFD}'))
}