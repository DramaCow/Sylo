use crate::lang::re;

// std::fs::write("_graph.dot", nfa.dot()).unwrap();

#[test]
fn simple_lexer() {
    let lexer = lexer_def! {
        [skip] _ws: re::any(" ,").plus(),
        word:       re::any("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz").plus()
    }.compile();
            
    let tokens = lexer.scan("Waltz, bad nymph, for quick jigs vex").collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(tokens[0].lexeme, "Waltz");
    assert_eq!(tokens[1].lexeme, "bad");
    assert_eq!(tokens[2].lexeme, "nymph");
    assert_eq!(tokens[3].lexeme, "for");
    assert_eq!(tokens[4].lexeme, "quick");
    assert_eq!(tokens[5].lexeme, "jigs");
    assert_eq!(tokens[6].lexeme, "vex");
    assert!(tokens.iter().all(|token| token.class == 1));
}