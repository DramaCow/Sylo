use crate::lang::re;

// std::fs::write("_graph.dot", nfa.dot()).unwrap();

#[test]
fn simple_lexer() {
    let lexer = lexer_def! {
        [skip] _ws: re::any(" ,").plus(),
        word:       re::any("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz").plus()
    }.compile();
    
    let text = "Waltz, bad nymph, for quick jigs vex";

    let tokens = lexer.scan(text).collect::<Result<Vec<_>, _>>().unwrap();

    assert_eq!(&text[tokens[0].span.clone()], "Waltz");
    assert_eq!(&text[tokens[1].span.clone()], "bad");
    assert_eq!(&text[tokens[2].span.clone()], "nymph");
    assert_eq!(&text[tokens[3].span.clone()], "for");
    assert_eq!(&text[tokens[4].span.clone()], "quick");
    assert_eq!(&text[tokens[5].span.clone()], "jigs");
    assert_eq!(&text[tokens[6].span.clone()], "vex");
    assert!(tokens.iter().all(|token| token.class == 1));
}