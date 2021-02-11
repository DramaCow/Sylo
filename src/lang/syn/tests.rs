use std::iter::once;

#[test]
fn parentheses_grammar() {
    let parser = syn_def! {
        { open, close }
        List : List Pair
             | Pair,
        Pair : open List close
             | open close,
    }.1.compile().unwrap();

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
        let parse = parser.parse(input.iter().cloned()).collect::<Result<Vec<_>, _>>();

        assert!(parse.is_ok() == valid, 
            "Input {:?} is {}",
            input.iter().map(|i| (&["(", ")"])[*i]).collect::<String>(),
            if valid { "valid" } else { "invalid" }
        );
    }
}

#[test]
fn parentheses_grammar_2() {
    let parser = syn_def! {
        { open, close }
        List : List Pair
             | Pair,
        Pair : open List close
             | open close,
    }.1.compile().unwrap();

    let input = vec![0, 0, 1, 1];

    let actions = parser.parse(input.iter().cloned()).collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(actions[0], super::Instruction::Shift { word: 0, index: 0 });
    assert_eq!(actions[1], super::Instruction::Shift { word: 0, index: 1 });
    assert_eq!(actions[2], super::Instruction::Shift { word: 1, index: 2 });
    assert_eq!(actions[3], super::Instruction::Reduce { var: 1, count: 2 });
    assert_eq!(actions[4], super::Instruction::Reduce { var: 0, count: 1 });
    assert_eq!(actions[5], super::Instruction::Shift { word: 1, index: 3 });
    assert_eq!(actions[6], super::Instruction::Reduce { var: 1, count: 3 });
    assert_eq!(actions[7], super::Instruction::Reduce { var: 0, count: 1 });
    
    // println!("{:?}", parse.unwrap());
}

// #[test]
// fn parse_tree() {
//     let (lex_grammar, syn_grammar) = crate::meta::make_grammars();

//     let var_names  = &[ "Expr", "Term", "Value", "Factor", "__bar_Term_star__",
//                         "__Value_plus__", "Set", "CSet", "Items", "Item",
//                         "Range", "__Items_plus__", "__start__" ];
//     let word_names = &["whitespace", "literal", "|", "?", "*", "+", "(", ")", "[", "]", "^", "-", "char"];

//     let lexer  = Lexer::compile(&lex_grammar).unwrap();
//     let parser = Parser::compile(&syn_grammar).unwrap();

//     println!("{}", debug::format_grammar(&syn_grammar, var_names, word_names));
//     std::fs::write("_parser.txt", parser.dumps()).unwrap();

//     let tokens = lexer.scan("[A-Za-z] [A-Za-z0-9_]   *").collect::<Result<Vec<_>, _>>().unwrap();
//     println!("{}", tokens.iter().map(|token| word_names[token.class]).collect::<Vec<_>>().join(" "));

//     let tree = parser.parse_tree(tokens.iter().map(|token| token.class)).unwrap();
//     std::fs::write("_tree.dot", debug::dot_parse_tree(&tree, var_names, word_names)).unwrap();
// }

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