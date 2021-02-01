// #![allow(dead_code)]
// #![allow(non_snake_case)]

mod string_builder;
pub use self::string_builder::StringBuilder;

// use crate::cfg::{Grammar, Symbol, Symbol::Terminal as Word, Symbol::Variable as Var};
// use crate::cfg::first::First;
// use crate::cfg::lr1::{DFA, Item};

// use crate::syn::{SynAnalyzer, Action};
// // use crate::syn::cst::{CST, Node};

// #[must_use]
// pub fn format_alt(alt: &[Symbol], var_names: &[&str], word_names: &[&str]) -> String {
//     if alt.is_empty() {
//         "\u{03B5}".to_string()
//     } else {
//         alt.iter().map(|symbol| {
//             match symbol {
//                 Word(a) => word_names[*a],
//                 Var(A)  => var_names[*A],
//             }
//         }).collect::<Vec<_>>().join(" ")
//     }
// }

// #[must_use]
// pub fn format_grammar(grammar: &Grammar, var_names: &[&str], word_names: &[&str]) -> String {
//     let padding = var_names.iter().map(|var| var.len()).max().unwrap();
//     // let delim = " | ";
//     let delim = format!("\n{:padding$}| ", "", padding = padding + "::=".len()); let delim = delim.as_str();

//     grammar.rules().enumerate().map(|(A, rule)| {
//         format!("{variable:<padding$} ::= {rule}",
//             variable = var_names[A],
//             padding = padding,
//             rule = rule.alts().map(|alt| { format_alt(alt, var_names, word_names) }).collect::<Vec<_>>().join(delim)
//         )
//     }).collect::<Vec<_>>().join("\n")
// }

// #[must_use]
// pub fn format_first(grammar: &Grammar, first: &First, var_names: &[&str], word_names: &[&str]) -> String {
//     let format_set = |symbol: &Option<Symbol>| {
//         first.get(symbol).iter().map(|class| {
//             class.map_or("\u{03B5}", |a| word_names[a])
//         }).collect::<Vec<_>>().join(", ")
//     };

//     // let word_padding = word_names.iter().map(|var| var.len()).max().unwrap();
//     let var_padding  = var_names.iter().map(|var| var.len()).max().unwrap();

//     vec![
//         // format!("first(\u{03B5}) = {{{}}}", format_set(&Eps)),
//         // (0..grammar.termcount).map(|a| {
//         //     format!("first({variable:<padding$}) = {{{set}}}",
//         //         variable = word_names[a],
//         //         padding = word_padding,
//         //         set = format_set(&Word(a))
//         //     )
//         // }).collect::<Vec<_>>().join("\n"),
//         grammar.rules().enumerate().map(|(A, _)| {
//             format!("{variable:<padding$}  :  {set}",
//                 variable = var_names[A],
//                 padding = var_padding,
//                 set = format_set(&Some(Var(A))))
//         }).collect::<Vec<_>>().join("\n")
//     ].join("\n")
// }

// // ===

// #[must_use]
// pub fn latex_parser(parser: &SynAnalyzer, var_names: &[&str], word_names: &[&str]) -> String {
//     let ncol_action = parser.term_count() + 1;
//     let ncol_goto   = parser.var_count();

//     let mut output = StringBuilder::new();

//     output.writeln(&format!("\\begin{{tabular}}{{l||{}|{}}}", (0..ncol_action).map(|_| "c").collect::<String>(), (0..ncol_goto).map(|_| "c").collect::<String>()));
//     output.indent();
//     output.writeln(&format!("state & \\texttt{{\\$}} & {} & {} \\\\", word_names.join(" & "), var_names[..var_names.len() - 1].join(" & ")));
//     output.writeln("\\hline");

//     for (row , (actions, gotos)) in parser.actions().chunks_exact(ncol_action).zip(parser.gotos().chunks_exact(ncol_goto)).enumerate() {
//         output.writeln(&format!("{} & {} & {} \\\\",
//             row,
//             actions.iter().map(|action| {
//                 match action {
//                     Action::Invalid => "-".to_string(),
//                     Action::Accept => "acc".to_string(),
//                     Action::Shift(next_state) => format!("s {}", next_state),
//                     Action::Reduce(alt) => format!("r {}", alt),
//                 }
//             }).collect::<Vec<_>>().join(" & "),
//             gotos.iter().map(|goto| {
//                 goto.map_or("-".to_string(), |state| state.to_string())
//             }).collect::<Vec<_>>().join(" & ")
//         ));
//     }

//     output.unindent();
//     output.writeln("\\end{tabular}");

//     output.build()
// }