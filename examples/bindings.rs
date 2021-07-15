// #![allow(unused_imports)]

extern crate sylo;

use sylo::re;

fn main() {
    let text = "('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')* - '_'+";

    let tokens: Vec<_> = re::scan(text).collect::<Result<_,_>>().unwrap();

    for token in tokens {
        println!("{}\t{:?}", &text[token.span], token.ttype);
    }
}