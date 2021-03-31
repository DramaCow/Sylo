use crate::lang::cfg::{
    GrammarBuilder,
    Symbol::Terminal as Word,
    Symbol::Variable as Var,
};
use super::ArrayParsingTable;
use crate::lang::lr::{ParseTreeNode, Parse};

use std::iter::once;

#[test]
fn parentheses_grammar() {
    let grammar = GrammarBuilder::new()
        .rule(&[&[Var(0), Var(1)], &[Var(1)]])
        .rule(&[&[Word(0), Var(0), Word(1)], &[Word(0), Word(1)]])
        .try_build().unwrap();

    let parser = ArrayParsingTable::new(&grammar).unwrap();

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

    for sentence in all_sentences(2, 12) {
        let input = sentence.iter().cloned().map(Ok::<_,()>);
        let valid = is_valid(&sentence);
        let parse = Parse::new(&parser, input, |a: &usize| *a).collect::<Result<Vec<_>, _>>();

        assert!(parse.is_ok() == valid, 
            "Input {:?} is {}",
            sentence.iter().map(|&i| (&["(", ")"])[i]).collect::<String>(),
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

    let parser = ArrayParsingTable::new(&grammar).unwrap();

    let input = vec![0, 0, 1, 1].into_iter().map(Ok::<_,()>);

    let nodes = Parse::new(&parser, input, |a: &usize| *a).collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(nodes[0], ParseTreeNode::Word(0));
    assert_eq!(nodes[1], ParseTreeNode::Word(0));
    assert_eq!(nodes[2], ParseTreeNode::Word(1));
    assert_eq!(nodes[3], ParseTreeNode::Var { var: 1, child_count: 2 });
    assert_eq!(nodes[4], ParseTreeNode::Var { var: 0, child_count: 1 });
    assert_eq!(nodes[5], ParseTreeNode::Word(1));
    assert_eq!(nodes[6], ParseTreeNode::Var { var: 1, child_count: 3 });
    assert_eq!(nodes[7], ParseTreeNode::Var { var: 0, child_count: 1 });
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