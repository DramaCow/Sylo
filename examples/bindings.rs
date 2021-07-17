// #![allow(unused_imports)]

extern crate sylo;

use sylo::re;

fn main() {
    let text = "('A'..'Z' | '_')+";

    let tokens: Vec<_> = re::scan(text).collect::<Result<_,_>>().unwrap();

    for token in tokens {
        println!("{}\t{:?}", &text[token.span.0..token.span.1], token.ttype);
    }

    println!("{:?}", re::parse(text));
}