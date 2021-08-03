
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

    let parser = codegen::ir::Parser::new("Arithmetic", &def, &strategy::LR1).unwrap();
    std::fs::write("src/parsing/re.rs", codegen::RustWriter::new(String::new()).parser(&parser).unwrap().build()).unwrap();
}