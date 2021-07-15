// #![allow(unused_imports)]

#[macro_use] extern crate sylo;

use sylo::re;

fn main() {
    let text = "('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')* - '_'+";
    let scan = re::scan(text);
}