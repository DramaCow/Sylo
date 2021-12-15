extern crate sylo;

use sylo::tok::Scan;
use sylo::meta::GrammarParser;

fn main() {
    let text = include_str!("meta.grammar");
    let grammar = GrammarParser::new().parse(text, Scan::new(text)).unwrap();
    print!("{}", grammar.unparse());
}