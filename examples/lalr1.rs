
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

    let direct_read = builder.direct_read();
    let reads = builder.reads();
    let read = builder.read();
    let includes = builder.includes();
    let follow = builder.follow();

    let lookahead = builder.lookahead();

    let mut table = "nonterminal transition\tDR\tRead\tFollow\treads\tincludes\n".to_string();

    for (i, _) in builder.nonterminal_transitions().iter().enumerate() {
        table.push_str(&transition_names[i]);
        table.push('\t');
        table.push_str(&format_indices(&direct_read[i], &word_names));
        table.push('\t');
        table.push_str(&format_indices(&read[i], &word_names));
        table.push('\t');
        table.push_str(&format_indices(&follow[i], &word_names));
        table.push('\t');
        table.push_str(&format_indices(&reads[i], &transition_names));
        table.push('\t');
        table.push_str(&format_indices(&includes[i], &transition_names));
        table.push('\n');
    }

    std::fs::write("_lalr1.csv", table).unwrap();

    println!("Regex lexer-parser compiled in {:?}.", timer.elapsed());
}

fn format_indices<'a, I: IntoIterator<Item = &'a usize>, L: std::fmt::Display>(indices: I, labels: &[L]) -> String {
    indices.into_iter().map(|&t| format!("{}", labels[t])).collect::<Vec<_>>().join(", ")
}