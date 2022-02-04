extern crate sylo;

use sylo::tok::Scan;
use sylo::meta::MetaParser;
use sylo::repr::MetaRepr;

fn main() {
    let text = include_str!("meta.slang");
    let grammar = MetaParser::new().parse(text, Scan::new(text)).unwrap();
    println!("{}", grammar.dumps());
    println!("{}", grammar.repr().dumps());
}