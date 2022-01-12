extern crate sylo;

use sylo::tok::Scan;

fn main() {
    let text = include_str!("meta.grammar");

    for token in Scan::new(text) {
        println!("{:?}", token.unwrap());
    }
}