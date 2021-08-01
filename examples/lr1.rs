
#[macro_use] extern crate sylo;

use sylo::langcore::{re, lr::LR1A};
use sylo::codegen;
use sylo::langcore::lr1_table::LR1TableConstruction;
use sylo::parser::strategy;

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

    let lr1a = LR1A::new(&def.grammar);
    std::fs::write("_graph.dot", lr1a.dot(&def.grammar, &["id", "(", ")", "+", "*"], &def.var_names).unwrap()).unwrap();
    // std::fs::write("src/parsing/re.rs", codegen::rep::LR1Parser::new("RegEx", &def, &strategy::LR1).unwrap().to_rust(String::new()).unwrap()).unwrap();
}