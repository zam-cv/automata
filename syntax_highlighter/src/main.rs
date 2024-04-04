use analyzer::Analyzer;
use std::fs;

mod analyzer;
const GRAMMAR: &'static str = include_str!("grammar.txt");

fn main() -> anyhow::Result<()> {
    let analyzer = Analyzer::new(GRAMMAR, "program")?;
    analyzer.validate()?;

    let input = fs::read_to_string("src/test.txt")?;
    let tokens = analyzer.lexer(&input);
    println!("{:?}", tokens);

    Ok(())
}
