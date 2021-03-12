use crate::lang::cfg::{
    GrammarBuilder,
    Symbol::Terminal as Word,
    Symbol::Variable as Var,
};
use super::UncompressedTable;
use crate::lang::lr;

use std::iter::once;

#[test]
fn parentheses_grammar() {
    let grammar = GrammarBuilder::new()
        .rule(&[&[Var(0), Var(1)], &[Var(1)]])
        .rule(&[&[Word(0), Var(0), Word(1)], &[Word(0), Word(1)]])
        .try_build().unwrap();

    let parser = UncompressedTable::new(&grammar).unwrap();

    // ad hoc ground truth
    let is_valid = |input: &[usize]| -> bool {
        if input.is_empty() {
            false
        } else {
            let mut counter = 0_isize;
            for word in input {
                if *word == 0 {
                    counter += 1;
                } else {
                    counter -= 1;
                    if counter < 0 {
                        return false;
                    }
                }
            }
            counter == 0
        }
    };

    for input in &all_sentences(2, 12) {
        let valid = is_valid(input);
        let parse = lr::Parse::new(&parser, input.iter().cloned()).collect::<Result<Vec<_>, _>>();

        assert!(parse.is_ok() == valid, 
            "Input {:?} is {}",
            input.iter().map(|i| (&["(", ")"])[*i]).collect::<String>(),
            if valid { "valid" } else { "invalid" }
        );
    }
}

#[test]
fn parentheses_grammar_2() {
    let grammar = GrammarBuilder::new()
        .rule(&[&[Var(0), Var(1)], &[Var(1)]])
        .rule(&[&[Word(0), Var(0), Word(1)], &[Word(0), Word(1)]])
        .try_build().unwrap();

    let parser = UncompressedTable::new(&grammar).unwrap();

    let input = vec![0, 0, 1, 1];

    let actions = lr::Parse::new(&parser, input.iter().cloned()).collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(actions[0], lr::Node::Word { word: 0, index: 0 });
    assert_eq!(actions[1], lr::Node::Word { word: 0, index: 1 });
    assert_eq!(actions[2], lr::Node::Word { word: 1, index: 2 });
    assert_eq!(actions[3], lr::Node::Var { var: 1, child_count: 2 });
    assert_eq!(actions[4], lr::Node::Var { var: 0, child_count: 1 });
    assert_eq!(actions[5], lr::Node::Word { word: 1, index: 3 });
    assert_eq!(actions[6], lr::Node::Var { var: 1, child_count: 3 });
    assert_eq!(actions[7], lr::Node::Var { var: 0, child_count: 1 });
}

// =================
// === UTILITIES ===
// =================

fn all_sentences(termcount: usize, maxlen: usize) -> Vec<Vec<usize>> {
    (0..=maxlen).scan(vec![Vec::new()], |ws, _| {
        let res = ws.clone();
        *ws = ws.iter().flat_map(|w| (0..termcount).map(move |b| { w.iter().cloned().chain(once(b)).collect() })).collect();
        Some(res)
    }).flatten().collect()
}