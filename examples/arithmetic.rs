
#[macro_use] extern crate sylo;

use std::time::Instant;
use sylo::langcore::re;
use sylo::parser::strategy;
use sylo::codegen;

fn main() {
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

    std::fs::write("examples/arithmetic_parser.rs", codegen::rep::LR1Parser::new("Arithmetic", &def, &strategy::LR1).unwrap().to_rust(String::new()).unwrap()).unwrap();
}