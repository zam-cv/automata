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

struct Analyzer<'a> {
    pub initial_rule: &'a str,
    pub grammar: HashMap<&'a str, Vec<Vec<Expression<'a>>>>,
}

impl<'a> Analyzer<'a> {
    fn new(grammar: &'a str, initial_rule: &'a str) -> anyhow::Result<Self> {
        let mut map = HashMap::new();

        for (i, line) in grammar.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, " = ");

            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                let mut options = Vec::new();

                for option in value.split(" | ") {
                    let mut expressions = Vec::new();

                    for component in option.split(" ~ ") {
                        let component = component.trim();

                        if component.len() > 0 {
                            if component.starts_with("\"") {
                                expressions
                                    .push(Expression::String(&component[1..component.len() - 1]));
                                continue;
                            } else if INTERNAL_RULE.contains_key(component) {
                                expressions.push(Expression::InternalRule(component));
                                continue;
                            } else {
                                if component.ends_with("_keyword") {
                                    expressions.push(Expression::Keyword(component));
                                } else {
                                    expressions.push(Expression::Rule(component));
                                }
                                continue;
                            }
                        }

                        return Err(anyhow::anyhow!("Invalid grammar at line {}", i + 1));
                    }

                    options.push(expressions);
                }

                map.insert(key, options);
            } else {
                return Err(anyhow::anyhow!("Invalid grammar at line {}", i + 1));
            }
        }

        Ok(Self {
            grammar: map,
            initial_rule,
        })
    }

    fn validate(&self) -> anyhow::Result<()> {
        let mut visited = HashSet::new();

        for (_, options) in self.grammar.iter() {
            for option in options {
                for expression in option {
                    match expression {
                        Expression::Rule(rule) | Expression::Keyword(rule) => {
                            visited.insert(rule);

                            if !self.grammar.contains_key(rule) {
                                return Err(anyhow::anyhow!("Invalid rule: {}", rule));
                            }
                        }
                        Expression::InternalRule(rule) => {
                            if !INTERNAL_RULE.contains_key(rule) {
                                return Err(anyhow::anyhow!("Invalid internal rule: {}", rule));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        for rule in self.grammar.keys() {
            if !visited.contains(rule) && rule != &self.initial_rule {
                return Err(anyhow::anyhow!("Unused rule: {}", rule));
            }

            if rule.ends_with("_keyword")
                && (self.grammar[rule].len() != 1 || self.grammar[rule][0].len() != 1)
            {
                return Err(anyhow::anyhow!("Invalid keyword rule: {}", rule));
            }
        }

        Ok(())
    }

    fn lexer(&self, rule: &'a str, start: &mut usize, input: &'a str) -> Vec<Expression<'a>> {
        let mut tokens = Vec::new();

        if let Some(options) = self.grammar.get(rule) {
            'options: for option in options {
                let mut local_start = *start;
                let mut temp_tokens = Vec::new();

                for expression in option {
                    match expression {
                        Expression::String(string) => {
                            if input[local_start..].starts_with(string) {
                                temp_tokens.push(Expression::String(
                                    &input[local_start..local_start + string.len()],
                                ));
                                local_start += string.len();
                            } else {
                                continue 'options;
                            }
                        }
                        Expression::Rule(r) => {
                            let mut temp = self.lexer(r, &mut local_start, input);

                            if local_start == *start {
                                continue 'options;
                            }

                            temp_tokens.append(&mut temp);
                        }
                        Expression::InternalRule(rule) => {
                            if rule == &"EMPTY" {
                                continue;
                            }

                            let mut end = local_start;

                            while end < input.len() {
                                if INTERNAL_RULE[rule](&input[end..end + 1]) {
                                    end += 1;
                                } else {
                                    break;
                                }
                            }

                            if local_start == end {
                                continue 'options;
                            }

                            temp_tokens.push(Expression::String(&input[local_start..end]));
                            local_start = end;
                        }
                        Expression::Keyword(rule) => {
                            if let Expression::String(string) = &self.grammar[rule][0][0] {
                                if input[local_start..].starts_with(string) {
                                    temp_tokens.push(Expression::Keyword(
                                        &input[local_start..local_start + string.len()],
                                    ));
                                    local_start += string.len();
                                } else {
                                    continue 'options;
                                }
                            }
                        }
                    }
                }

                *start = local_start;
                tokens.append(&mut temp_tokens);
                return tokens;
            }
        }

        return tokens;
    }
}

fn main() -> anyhow::Result<()> {
    let input = fs::read_to_string("src/test.txt").unwrap();

    let analyzer = Analyzer::new(GRAMMAR, "program")?;
    analyzer.validate()?;

    // for (key, value) in analyzer.grammar.iter() {
    //     println!("{}: {:?}", key, value);
    // }

    let tokens = analyzer.lexer("program", &mut 0, &input);
    println!("{:?}", tokens);

    Ok(())
}
