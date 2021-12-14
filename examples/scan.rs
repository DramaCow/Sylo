extern crate sylo;

use sylo::tok::Scan;

fn main() {
    let input = "\"rule\" n:IDENT \"=\" e:Expr { Rule { name: n, expr: e } }";
    for token in Scan::new(input) {
        println!("{:?}", token.unwrap());
    }
}