
#[macro_use] extern crate sylo;

use sylo::langcore::{re, lr::LR1A};
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

    let lr1a = LR1A::new(&def.grammar);
    std::fs::write("_graph.dot", lr1a.dot(&def.grammar, &["id", "(", ")", "+", "*"], &def.var_names).unwrap()).unwrap();
    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}