
#[macro_use] extern crate sylo;

use sylo::lang::{
    re,
    cfg::First,
    lr::LR1ABuilder,
};
use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let def = parser! {
        {
            id:     re::range('a', 'z').plus(),
            lparen: re::literal("("),
            rparen: re::literal(")"),
            add:    re::literal("+"),
            mul:    re::literal("*"),
        },
        {
            E : E add T
              | T,
            T : T mul F
              | F,
            F : lparen E rparen
              | id,
        }
    };

    let lr1a = LR1ABuilder::new(&def.grammar, &First::new(&def.grammar)).build();
    std::fs::write("_graph.dot", lr1a.dot(&def.grammar, &["id", "(", ")", "+", "*"], &def.var_names, true).unwrap()).unwrap();
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}