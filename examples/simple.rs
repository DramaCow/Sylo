
#[macro_use] extern crate sylo;

use sylo::lang::re;

use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let def = parser_def! {
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
                      | word
        }
    };

    let parser = def.build().unwrap();

    let text = "never gonna give you up. never gonna let you down...";

    let cst = parser.cst(text).unwrap();
    std::fs::write("_graph.dot", cst.dot(&parser).unwrap()).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}