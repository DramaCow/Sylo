extern crate sylo;

use sylo::tok::Scan;
use sylo::meta::GrammarParser;
use sylo::bnf::BNF;

fn main() {
    let text = include_str!("meta.grammar");
    let grammar = GrammarParser::new().parse(text, Scan::new(text)).unwrap();
    let bnf = BNF::new(&grammar);
    print!("{}", bnf.dump());
}