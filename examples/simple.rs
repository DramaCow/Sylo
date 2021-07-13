
#[macro_use] extern crate sylo;

use std::time::Instant;
use sylo::lang::re;
use sylo::parser::strategy;
use sylo::codegen;

fn main() {
    let timer = Instant::now();

    let def = parser! {
        {
            [skip] _ws: re::any(" ,").plus(),
            word:       re::range('a', 'z').plus(),
            period:     re::literal("."),
            ellipses:   re::literal("..."),
        },
        {
            Paragraph : Sentences,
            Sentences : Sentences Sentence
                      | Sentence,
            Sentence  : Words period
                      | Words ellipses,
            Words     : Words word
                      | word,
        }
    };

    let parser = def.build(strategy::LR1).unwrap();

    let text = "never gonna give you up. never gonna let you down...";

    let cst = parser.cst(text).unwrap();
    std::fs::write("_graph.dot", cst.dot(&parser).unwrap()).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());

    std::fs::write("lexer.c.txt", codegen::c::lexer(String::new(), "MyLexer", &parser.lexer).unwrap()).unwrap();
}