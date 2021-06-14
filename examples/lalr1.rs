
#[macro_use] extern crate sylo;

use sylo::lang::{
    re,
    lr::LR0ABuilder,
    lr::LALR1ABuilder,
};
use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let def = parser! {
        {
            a: re::literal("a"),
        },
        {
            A : B C D A
              | a,
            B : ,
            C : ,
            D : ,
        }
    };
    
    let word_names = ["a"];

    let lr0a = LR0ABuilder::new(&def.grammar).build();
    std::fs::write("_graph.dot", lr0a.dot(&def.grammar, &word_names, &def.var_names, true).unwrap()).unwrap();

    let builder = LALR1ABuilder::new(&def.grammar);
    let transition_names: Vec<_> = builder.nonterminal_transitions().iter()
        .map(|&transition| format!("(s{}, {})", transition.state, def.var_names[transition.var])).collect();

    for (i, dr) in builder.direct_read().iter().enumerate() {
        println!("DR{} = {{{}}}", transition_names[i], format_indices(dr.iter().copied(), &word_names));
    }

    let mut reads = builder.reads_relation();

    for i in 0..builder.nonterminal_transitions().len() {
        let related = format_indices(reads(i), &transition_names);
        println!("{} reads {{{}}}", transition_names[i], related);
    }

    for (i, read) in builder.read().iter().enumerate() {
        println!("Read{} = {{{}}}", transition_names[i], format_indices(read.iter().copied(), &word_names));
    }

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}

fn format_indices<I: IntoIterator<Item = usize>, L: std::fmt::Display>(indices: I, labels: &[L]) -> String {
    indices.into_iter().map(|t| format!("{}", labels[t])).collect::<Vec<_>>().join(", ")
}