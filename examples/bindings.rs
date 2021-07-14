// #![allow(unused_imports)]

#[macro_use] extern crate sylo;

use sylo::bindings;

fn main() {
    let text = "('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')* - '_'+".as_bytes();
    let scan = unsafe { bindings::RegEx_Lexer_new(text.as_ptr(), text.len() as ::std::os::raw::c_ulong); };
}