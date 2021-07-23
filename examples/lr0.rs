#[macro_use] extern crate sylo;

use sylo::langcore::{re, lr::LR0A};
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

    let lr0a = LR0A::new(&def.grammar);
    std::fs::write("_graph.dot", lr0a.dot(&def.grammar, &["id", "(", ")", "+", "*"], &def.var_names).unwrap()).unwrap();
    // pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> String

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}