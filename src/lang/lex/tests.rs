use crate::lang::re;
use crate::lang::lex::LexAnalyzer;
use super::parse::Token;

// std::fs::write("_graph.dot", nfa.dot()).unwrap();

#[test]
fn simple_lexer() {
    let lexer = LexAnalyzer::compile(&lex_def! {
        [skip] _ws: re::any(" ,").plus(),
        word:       re::any("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz").plus()
    });
            
    let tokens = lexer.parse("Waltz, bad nymph, for quick jigs vex").collect::<Result<Vec<_>, _>>().unwrap();
    
    assert_eq!(tokens[0], Token { lexeme: "Waltz", class: 1 });
    assert_eq!(tokens[1], Token { lexeme: "bad",   class: 1 });
    assert_eq!(tokens[2], Token { lexeme: "nymph", class: 1 });
    assert_eq!(tokens[3], Token { lexeme: "for",   class: 1 });
    assert_eq!(tokens[4], Token { lexeme: "quick", class: 1 });
    assert_eq!(tokens[5], Token { lexeme: "jigs",  class: 1 });
    assert_eq!(tokens[6], Token { lexeme: "vex",   class: 1 });
}