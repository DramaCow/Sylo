
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
            [skip] _ws: re::any(" \n\t\r").plus(),
            string:     re::literal("\"").then(&c.plus()).then(&re::literal("\"")),
            character:  re::literal("'").then(&c).then(&re::literal("'")),
            range:      re::literal(".."),
            and:        re::literal("&"),
            or:         re::literal("|"),
            diff:       re::literal("-"),
            opt:        re::literal("?"),
            star:       re::literal("*"),
            plus:       re::literal("+"),
            not:        re::literal("!"),
            open:       re::literal("("),
            close:      re::literal(")"),
        },
        parser: {
            // You can skip the first node if desired.
            [skip] Expr_ : AltOrConjOrSeqOrUnaryOrFactor_,

            Alt         : ConjOrSeqOrUnaryOrFactor_ AltPlus_,
            Conj        : SeqOrUnaryOrFactor_ ConjPlus_,
            Seq         : UnaryOrFactor_ SeqPlus_,
            Unary       : Factor_ opt
                        | Factor_ star
                        | Factor_ plus
                        | not Factor_,
            Parentheses : open AltOrConjOrSeqOrUnaryOrFactor_ close,
            Range       : character range character,

            // dummy variables
            [skip] AltPlus_                       : AltPlus_ or ConjOrSeqOrUnaryOrFactor_
                                                  | or ConjOrSeqOrUnaryOrFactor_,
            [skip] ConjPlus_                      : ConjPlus_ ConjOp_
                                                  | ConjOp_,
            [skip] ConjOp_                        : and SeqOrUnaryOrFactor_
                                                  | diff SeqOrUnaryOrFactor_,
            [skip] SeqPlus_                       : SeqPlus_ UnaryOrFactor_
                                                  | UnaryOrFactor_,
            [skip] AltOrConjOrSeqOrUnaryOrFactor_ : Alt
                                                  | ConjOrSeqOrUnaryOrFactor_,
            [skip] ConjOrSeqOrUnaryOrFactor_      : Conj
                                                  | SeqOrUnaryOrFactor_,
            [skip] SeqOrUnaryOrFactor_            : Seq
                                                  | UnaryOrFactor_,
            [skip] UnaryOrFactor_                 : Unary
                                                  | Factor_,
            [skip] Factor_                        : Parentheses
                                                  | string
                                                  | character
                                                  | Range,
        }
    };

    let parser = def.compile().unwrap();

    let cst = parser.cst("(('A'..'Z' | 'a'..'z' | '_') ('A'..'Z' | 'a'..'z' | '0'..'9' | '_')*) - '_'+").unwrap();
    std::fs::write("_graph.dot", cst.dot_with_labelling(|var| &parser.syn_labels[var])).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}