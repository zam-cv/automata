use analyzer::Analyzer;
use std::fs;

mod analyzer;
mod parsed;

const GRAMMAR: &'static str = include_str!("grammar.txt");

fn main() -> anyhow::Result<()> {
    let analyzer = Analyzer::new(GRAMMAR, "program")?;
    analyzer.validate()?;

    let input = fs::read_to_string("src/test.txt")?;
    let parsed = analyzer.parse(&input);

    // example
    parsed.visit(&mut |chunk| match chunk.rule() {
        "string" => {
            if chunk.value() == ";" {
                println!("string: {}", chunk.value());
            }
        }
        _ => {}
    });

    Ok(())
}
