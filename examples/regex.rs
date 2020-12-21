
#[macro_use] extern crate sylo;

use sylo::lang::re;

use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let c = re::non_compatibility_char()
        .diff(&re::literal("\\"))
        .diff(&re::literal("\""))
        .or(&re::literal("\\\\"))
        .or(&re::literal("\\\""));

    let def = parser_def! {
        lexer: {
            [skip] _ws:   re::any(" \n\t\r").plus(),
            string:       re::literal("\"").then(&c.plus()).then(&re::literal("\"")),
            character:    re::literal("'").then(&c).then(&re::literal("'")),
            range:        re::literal(".."),
            and:          re::literal("&"),
            or:           re::literal("|"),
            diff:         re::literal("-"),
            opt:          re::literal("?"),
            star:         re::literal("*"),
            plus:         re::literal("+"),
            not:          re::literal("!"),
            open:         re::literal("("),
            close:        re::literal(")"),
        },
        parser: {
            Expr                : Term AltTermStar_,
            Term                : ValuePlus_,
            Value               : Factor opt
                                | Factor star
                                | Factor plus
                                | not Factor
                                | Factor,
            Factor              : open Expr close
                                | string
                                | character
                                | character range character,
            [skip] AltTermStar_ : AltTermStar_ or Term
                                | ,
            [skip] ValuePlus_   : ValuePlus_ Value
                                | Value,
        }
    };

    let _parser = sylo::lang::parser::Parser::try_compile(&def).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}