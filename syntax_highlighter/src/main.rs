use analyzer::{Analyzer, Token};
use std::fs;

mod analyzer;
const GRAMMAR: &'static str = include_str!("grammar.txt");

fn main() -> anyhow::Result<()> {
    let analyzer = Analyzer::new(GRAMMAR, "program")?;
    analyzer.validate()?;

    let input = fs::read_to_string("src/test.txt")?;
    let response = analyzer.parser(&input);

    analyzer.visit(&response.0, &mut |Token(rule, others)| match *rule {
        "ident" => {
            println!("Ident: {}", others.value(&input));
        }
        _ => {}
    });

    Ok(())
}
