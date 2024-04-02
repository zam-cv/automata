use lazy_static::lazy_static;
use std::{
    collections::{HashMap, HashSet},
    fs,
};
const GRAMMAR: &'static str = include_str!("grammar.txt");

lazy_static! {
    static ref INTERNAL_RULE: HashMap<&'static str, fn(&str) -> bool> = {
        let mut map: HashMap<&str, fn(&str) -> bool> = HashMap::new();
        map.insert("ASCII_DIGIT", |s| s.chars().all(|c| c.is_ascii_digit()));
        map.insert("ASCII_ALPHA", |s| {
            s.chars().all(|c| c.is_ascii_alphabetic())
        });
        map.insert("ASCII_ALPHANUMERIC", |s| {
            s.chars().all(|c| c.is_ascii_alphanumeric())
        });
        map.insert("ANY", |s| s.chars().all(|c| c.is_ascii()));
        map.insert("WHITESPACE", |s| s.chars().all(|c| c.is_whitespace()));
        map.insert("EMPTY", |s| s.is_empty());
        map
    };
}

#[derive(Debug)]
enum Expression<'a> {
    Keyword(&'a str),
    String(&'a str),
    Rule(&'a str),
    InternalRule(&'a str),
}

fn get_graph(grammar: &str) -> anyhow::Result<HashMap<&str, Vec<Vec<Expression>>>> {
    let mut graph = HashMap::new();

    for (i, line) in grammar.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(2, " = ");

        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            let mut options = Vec::new();

            for option in value.split(" | ") {
                let mut components = Vec::new();

                for component in option.split(" ~ ") {
                    let component = component.trim();

                    if component.len() > 0 {
                        if component.starts_with("\"") {
                            components.push(Expression::String(&component[1..component.len() - 1]));
                            continue;
                        } else if INTERNAL_RULE.contains_key(component) {
                            components.push(Expression::InternalRule(component));
                            continue;
                        } else {
                            if component.ends_with("_keyword") {
                                components.push(Expression::Keyword(component));
                            } else {
                                components.push(Expression::Rule(component));
                            }
                            continue;
                        }
                    }

                    return Err(anyhow::anyhow!("Invalid grammar at line {}", i + 1));
                }

                options.push(components);
            }

            graph.insert(key, options);
        } else {
            return Err(anyhow::anyhow!("Invalid grammar at line {}", i + 1));
        }
    }

    Ok(graph)
}

fn validate_graph(
    initial_rule: &str,
    graph: &HashMap<&str, Vec<Vec<Expression>>>,
) -> anyhow::Result<()> {
    let mut visited = HashSet::new();

    for (_, options) in graph.iter() {
        for option in options {
            for expression in option {
                match expression {
                    Expression::Rule(rule) | Expression::Keyword(rule) => {
                        visited.insert(rule);

                        if !graph.contains_key(rule) {
                            return Err(anyhow::anyhow!("Invalid rule: {}", rule));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for rule in graph.keys() {
        if !visited.contains(rule) && rule != &initial_rule {
            return Err(anyhow::anyhow!("Unused rule: {}", rule));
        }
    }

    Ok(())
}

fn lexer<'a>(
    rule: &str,
    start: usize,
    input: &'a str,
    tokens: &mut Vec<Expression<'a>>,
    graph: &HashMap<&str, Vec<Vec<Expression>>>,
) {
    if let Some(options) = graph.get(rule) {
        for option in options {
            println!("{:?}", option);
        }
    }
}

fn analyzer(initial_rule: &str, grammar: &str, input: &str) -> anyhow::Result<()> {
    let graph = get_graph(grammar)?;
    validate_graph(initial_rule, &graph)?;

    for (key, value) in &graph {
        println!("{}: {:?}", key, value);
    }

    // let mut tokens = Vec::new();
    // lexer(initial_rule, 0, input, &mut tokens, &graph);
    // println!("{:?}", tokens);

    Ok(())
}

fn main() {
    let input = fs::read_to_string("src/test.txt").unwrap();

    if let Err(e) = analyzer("program", GRAMMAR, &input) {
        eprintln!("{}", e);
    }
}
