
#[macro_use] extern crate sylo;

use sylo::re;

use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let char_regex = re::non_compatibility_char();

    let def = parser_def! {
        lexer: {
            [skip] _ws:   re::any(" \n\t\r").plus(),
            literal:      re::literal("\"").then(&char_regex.and(&re::literal("\"").not()).plus()).then(&re::literal("\"")),
            alt:          re::literal("\\|"),
            opt:          re::literal("\\?"),
            star:         re::literal("\\*"),
            plus:         re::literal("\\+"),
            open_round:   re::literal("("),
            close_round:  re::literal(")"),
            open_square:  re::literal("["),
            close_square: re::literal("]"),
            hat:          re::literal("^"),
            dash:         re::literal("-"),
            character:    char_regex,
        },
        parser: {
            Expr                : Term AltTermStar_,
            Term                : ValuePlus_,
            Value               : Factor opt
                                | Factor star
                                | Factor plus
                                | Factor,
            Factor              : open_round Expr close_round
                                | Set
                                | CSet
                                | literal,
            [skip] AltTermStar_ : AltTermStar_ alt Term
                                | ,
            [skip] ValuePlus_   : ValuePlus_ Value
                                | Value,
            Set                 : open_square Items close_square,
            CSet                : open_square hat Items close_square,
            Items               : ItemsPlus_,
            Item                : Range
                                | character,
            Range               : character dash character,
            [skip] ItemsPlus_   : ItemsPlus_ Item
                                | Item,
        }
    };

    let _parser = sylo::parser::Parser::try_compile(&def).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}