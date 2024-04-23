use crate::parsed::Parsed;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

const ROOT: &'static str = "root";
const EMPTY: &'static str = "EMPTY";
const ASCII_DIGIT: &'static str = "ASCII_DIGIT";
const ASCII_ALPHA: &'static str = "ASCII_ALPHA";
const ASCII_ALPHANUMERIC: &'static str = "ASCII_ALPHANUMERIC";
const WHITESPACE: &'static str = "WHITESPACE";

lazy_static! {
    // reglas internas que se pueden utilizar en la gramatica por simplicidad
    static ref INTERNAL_RULE: HashMap<&'static str, fn(&str) -> bool> = {
        let mut map: HashMap<&str, fn(&str) -> bool> = HashMap::new();
        map.insert(ASCII_DIGIT, |s| s.chars().all(|c| c.is_ascii_digit()));
        map.insert(ASCII_ALPHA, |s| s.chars().all(|c| c.is_ascii_alphabetic()));
        map.insert(ASCII_ALPHANUMERIC, |s| {
            s.chars().all(|c| c.is_ascii_alphanumeric())
        });
        map.insert(WHITESPACE, |s| s.chars().all(|c| c.is_whitespace()));
        map.insert(EMPTY, |s| s.is_empty());
        map
    };
}

#[derive(Debug)]
pub struct Token<'a>(pub &'a str, pub ((usize, usize), Option<Vec<Token<'a>>>));

#[derive(Debug)]
pub struct Error<'a> {
    pub message: &'a str,
    pub first: usize,
    pub last: usize,
}

#[derive(Debug)]
pub enum Expression<'a> {
    Keyword(&'a str),
    String(&'a str),
    Rule(&'a str),
    InternalRule(&'a str),
}

pub struct Analyzer<'a> {
    pub initial_rule: &'a str,
    pub grammar: HashMap<&'a str, Vec<Vec<Expression<'a>>>>,
}

impl<'a> Analyzer<'a> {
    pub fn new(grammar: &'a str, initial_rule: &'a str) -> anyhow::Result<Self> {
        let mut map = HashMap::new();

        // se lee la gramatica y se almacena en un hashmap
        for (i, line) in grammar.lines().enumerate() {
            // se ignora las lineas vacias que se usan para separar las reglas
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
                            // se valida si es un string, una regla, una regla interna o una keyword
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

    // se usa en el desarrollo para validar la gramatica
    pub fn validate(&self) -> anyhow::Result<()> {
        let mut visited = HashSet::new();

        for (_, options) in self.grammar.iter() {
            for option in options {
                for expression in option {
                    // se valida que las reglas existan
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
            // se valida que todas las reglas sean usadas
            if !visited.contains(rule) && rule != &self.initial_rule {
                return Err(anyhow::anyhow!("Unused rule: {}", rule));
            }

            // se valida que las reglas keyword tengan un solo string
            if rule.ends_with("_keyword")
                && (self.grammar[rule].len() != 1 || self.grammar[rule][0].len() != 1)
            {
                return Err(anyhow::anyhow!("Invalid keyword rule: {}", rule));
            }
        }

        Ok(())
    }

    pub fn parse(&self, input: &'a str) -> Parsed {
        let mut position = 0;
        let mut errors = Vec::new();
        let mut tokens = Vec::new();

        // iterar sobre el input, si no se ha llegado al final se intenta parsear lo demas
        // nota: si no se llega al final del input, se asume que hay un error de sintaxis
        while position < input.len() {
            let mut tmp = position;
            // se parsea el input
            let mut token = self.resursive_parse(self.initial_rule, &mut tmp, &mut errors, input);

            if tmp + 1 < input.len() {
                if let Some(children) = &mut token.1 .1 {
                    // se tiene que adaptar a la regla a la que probablemente pertenece
                    children.push(Token("unknown", ((tmp, tmp + 1), None)));
                }    
            }

            tokens.push(token);
            position = tmp + 1;
        }

        Parsed::new(input, Token(ROOT, ((0, input.len()), Some(tokens))), errors)
    }

    fn resursive_parse(
        &self,
        rule: &'a str,
        start: &mut usize,
        errors: &mut Vec<Error<'a>>,
        input: &'a str,
    ) -> Token<'a> {
        let mut tokens = Vec::new();
        // se almacenan los candidatos por si alguna regla no se cumple
        // y tomar al que tenga mayor score tenga
        let mut candidates = Vec::new();

        if let Some(options) = self.grammar.get(rule) {
            // se itera sobre las opciones de la regla
            'options: for option in options {
                let mut score = 0;
                let mut local_start = *start;
                let mut temp_tokens = Vec::new();

                // se itera sobre las expresiones de la opcion
                // Nota: las expresiones pueden ser strings, reglas, reglas internas o keywords
                for expression in option {
                    match expression {
                        Expression::String(string) => {
                            if input[local_start..].starts_with(string) {
                                // se avanza la posicion y se aumenta el score
                                local_start += string.len();
                                score += 1;

                                temp_tokens.push(Token(
                                    "string",
                                    ((local_start - string.len(), local_start), None),
                                ));
                            } else {
                                // si no se cumple la regla se almacena como candidato
                                candidates.push((
                                    local_start,
                                    score as f32 / option.len() as f32,
                                    temp_tokens,
                                ));
                                continue 'options;
                            }
                        }
                        Expression::Rule(r) => {
                            // se llama recursivamente a la regla y se almacena el resultado
                            let temp = self.resursive_parse(r, &mut local_start, errors, input);

                            // si no se cumple la regla va a la siguiente opcion
                            if local_start == *start {
                                continue 'options;
                            }

                            score += 1;
                            temp_tokens.push(temp);
                        }
                        Expression::InternalRule(rule) => {
                            // se salta las reglas internas vacias
                            if rule == &EMPTY {
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
                                candidates.push((
                                    local_start,
                                    score as f32 / option.len() as f32,
                                    temp_tokens,
                                ));
                                continue 'options;
                            }

                            temp_tokens.push(Token("internal_rule", ((local_start, end), None)));
                            local_start = end;
                            score += 1;
                        }
                        Expression::Keyword(rule) => {
                            if let Expression::String(string) = &self.grammar[rule][0][0] {
                                if input[local_start..].starts_with(string) {
                                    temp_tokens.push(Token(
                                        "keyword",
                                        ((local_start, local_start + string.len()), None),
                                    ));
                                    local_start += string.len();
                                    score += 1;
                                } else {
                                    continue 'options;
                                }
                            }
                        }
                    }
                }

                // se da la posicion del token y se almacena
                let position = (*start, local_start);
                *start = local_start;
                tokens.append(&mut temp_tokens);
                return Token(rule, ((position.0, position.1), Some(tokens)));
            }
        }

        // si ninguna regla se cumple se toma el candidato con mayor score
        if let Some((local_start, _, temp_tokens)) = candidates
            .iter_mut()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        {
            // se sabe que no es una regla interna, ni un string, omite desviaciones y
            // escala hasta tener mayor informacion del error
            if temp_tokens.len() > 1 {
                errors.push(Error {
                    message: "Syntax error",
                    first: *start,
                    last: *local_start,
                });

                let position = (*start, *local_start);
                *start = *local_start;
                tokens.append(temp_tokens);
                return Token(rule, (position, Some(tokens)));
            }
        }

        // si no hay candidatos se asume que hay un error de sintaxis
        let position = (*start, *start);
        Token(rule, (position, Some(tokens)))
    }
}
