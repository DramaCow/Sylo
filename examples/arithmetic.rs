
#[macro_use] extern crate sylo;

use sylo::langcore::re;
use sylo::parser::strategy;
use sylo::codegen;

fn main() {
    let def = parser! {
        {
            num:    re::literal("-").opt().then(&re::range('1', '9')).then(&re::range('0', '9').star()).or(&re::literal("0")),
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
              | num,
        }
    };

    std::fs::write("examples/arithmetic_parser.rs", codegen::rep::LR1Parser::new("Arithmetic", &def, &strategy::LR1).unwrap().to_rust(String::new()).unwrap()).unwrap();
}