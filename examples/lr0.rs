
#[macro_use] extern crate sylo;

use sylo::lang::{
    re,
    lr1::LR0ABuilder,
};
use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let def = parser_def! {
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

    let lr0a = LR0ABuilder::new(&def.grammar).build();
    std::fs::write("_graph.dot", lr0a.dot(&def.grammar, &["id", "(", ")", "+", "*"], &def.var_names, true)).unwrap();
    // pub fn dot<T, U>(&self, grammar: &Grammar, word_names: &[T], var_names: &[U], print_itemsets: bool) -> String


    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}