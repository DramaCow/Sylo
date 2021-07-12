
#[macro_use] extern crate sylo;

use sylo::lang::{
    re,
    lr::{LALR1ABuilder, StateReductionPair},
};
use std::time::Instant;

fn main() {
    let timer = Instant::now();

    // let def = parser! {
    //     {
    //         a: re::literal("a"),
    //     },
    //     {
    //         A : B C D A
    //           | a,
    //         B : ,
    //         C : ,
    //         D : ,
    //     }
    // };
    // let word_names = ["a"];
    let def = parser! {
        {
            add:    re::literal("+"),
            id:     re::literal("n"),
            lparen: re::literal("("),
            rparen: re::literal(")"),
        },
        {
            E : E add T
              | T,
            T : id
              | lparen E rparen,
        }
    };
    let word_names = ["-", "n", "(", ")"];
    
    let lr0a = LALR1ABuilder::new(&def.grammar).build();
    std::fs::write("_graph.dot", lr0a.dot(&def.grammar, &word_names, &def.var_names).unwrap()).unwrap();
    
    let word_names = ["$", "-", "n", "(", ")"];
    
    let builder = LALR1ABuilder::new(&def.grammar);

    let transition_names: Vec<_> = builder.nonterminal_transitions().iter()
        .map(|&transition| format!("(s{}, {})", transition.state, def.var_names[transition.var])).collect();

    let direct_read = builder.direct_read();
    let reads = builder.reads();
    let read = builder.read();
    let includes = builder.includes();
    let follow = builder.follow();
    let lookback = builder.lookback();
    let lookahead = builder.lookahead();

    let mut table = "nonterminal transition\tDR\tRead\tFollow\treads\tincludes\n".to_string();

    for (i, _) in builder.nonterminal_transitions().iter().enumerate() {
        table.push_str(&transition_names[i]);
        table.push('\t');
        table.push_str(&format_indices(direct_read[i].iter().map(|o| o.map_or(0, |x| x + 1)), &word_names));
        table.push('\t');
        table.push_str(&format_indices(read[i].iter().map(|o| o.map_or(0, |x| x + 1)), &word_names));
        table.push('\t');
        table.push_str(&format_indices(follow[i].iter().map(|o| o.map_or(0, |x| x + 1)), &word_names));
        table.push('\t');
        table.push_str(&format_indices(reads[i].iter().copied(), &transition_names));
        table.push('\t');
        table.push_str(&format_indices(includes[i].iter().copied(), &transition_names));
        table.push('\n');
    }

    for (state_reduction_pair, lb) in &lookback {
        let StateReductionPair { state, production } = state_reduction_pair;
        println!("(s{}, p{}) lookback {{{}}}", state, production, format_indices(lb.iter().copied(), &transition_names));
    }

    for (state_reduction_pair, la) in &lookahead {
        let StateReductionPair { state, production } = state_reduction_pair;
        println!("LA(s{}, p{}) = {{{}}}", state, production, format_indices(la.iter().map(|o| o.map_or(0, |x| x + 1)), &word_names));
    }

    std::fs::write("_lalr1.csv", table).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}

fn format_indices<I: IntoIterator<Item = usize>, L: std::fmt::Display>(indices: I, labels: &[L]) -> String {
    let mut vec: Vec<_> = indices.into_iter().collect(); vec.sort();
    vec.into_iter().map(|t| format!("{}", labels[t])).collect::<Vec<_>>().join(", ")
}