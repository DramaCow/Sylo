
#[macro_use] extern crate sylo;

use sylo::lang::re;

use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let c = re::basic_multilingual_plane();

    let def = parser_def! {
        lexer: {
            [skip] _ws: re::any(" ,").plus(),
            word:       re::range('a', 'z').plus(),
            period:     re::literal("."),
            ellipses:   re::literal("..."),
        },
        parser: {
            Paragraph        : Sentences,
            [skip] Sentences : Sentences Sentence
                             | Sentence,
            Sentence         : Words period
                             | Words ellipses,
            [skip] Words     : Words word
                             | word
        }
    };

    std::fs::write("_dfa.dot", def.dot_lr1a()).unwrap();

    let parser = def.compile().unwrap();

    let cst = parser.cst("never gonna give you up. never gonna let you down...").unwrap();
    std::fs::write("_graph.dot", cst.dot(&parser)).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}