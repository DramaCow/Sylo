use regex_deriv as re;

// #[test]
// fn simple_lexer() {
//     let lexer = lexer! {
//         [skip] _ws: re::any(" ,").plus(),
//         word:       re::any("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz").plus()
//     }.build();
    
//     let text = "Waltz, bad nymph, for quick jigs vex";

//     let tokens: Vec<_> = lexer.scan(text).collect::<Result<_, _>>().unwrap();

//     assert_eq!(&text[tokens[0].span.clone()], "Waltz");
//     assert_eq!(&text[tokens[2].span.clone()], "bad");
//     assert_eq!(&text[tokens[4].span.clone()], "nymph");
//     assert_eq!(&text[tokens[6].span.clone()], "for");
//     assert_eq!(&text[tokens[8].span.clone()], "quick");
//     assert_eq!(&text[tokens[10].span.clone()], "jigs");
//     assert_eq!(&text[tokens[12].span.clone()], "vex");
//     assert!(tokens.iter().all(|token| token.class == 1));
// }

use crate::meta;

#[test]
fn calculator1() {
    assert!(meta::TermParser::new().parse("22").is_ok());
    assert!(meta::TermParser::new().parse("(22)").is_ok());
    assert!(meta::TermParser::new().parse("((((22))))").is_ok());
    assert!(meta::TermParser::new().parse("((22)").is_err());
}