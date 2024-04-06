use crate::analyzer::{Error, Token};

#[derive(Debug)]
pub struct Chunk<'a, 'b> {
    #[allow(dead_code)]
    input: &'a str,
    pub token: &'b Token<'a>,
}

pub struct Parsed<'a> {
    input: &'a str,
    pub token: Token<'a>,
    pub errors: Vec<Error<'a>>,
}

fn recursive_visit<'a, 'b, F>(input: &'a str, token: &'b Token<'a>, f: &mut F)
where
    F: FnMut(&Chunk<'a, 'b>),
{
    f(&Chunk { input, token });

    if let (_, Some(children)) = &token.1 {
        for child in children {
            recursive_visit(input, child, f);
        }
    }
}

impl<'a, 'b> Chunk<'a, 'b> {
    #[allow(dead_code)]
    pub fn rule(&self) -> &'a str {
        &self.token.0
    }

    #[allow(dead_code)]
    pub fn position(&self) -> &(usize, usize) {
        &self.token.1 .0
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &'a str {
        let position = self.position();
        &self.input[position.0..position.1]
    }

    #[allow(dead_code)]
    pub fn children(&self) -> Option<&Vec<Token<'a>>> {
        self.token.1 .1.as_ref()
    }

    #[allow(dead_code)]
    pub fn visit<'c, F>(&'c self, f: &mut F)
    where
        F: FnMut(&Chunk<'a, 'c>),
    {
        recursive_visit(self.input, self.token, f);
    }
}

impl<'a> Parsed<'a> {
    pub fn new(input: &'a str, token: Token<'a>, errors: Vec<Error<'a>>) -> Self {
        Parsed {
            input,
            token,
            errors,
        }
    }

    #[allow(dead_code)]
    pub fn visit<'b, F>(&'b self, f: &mut F)
    where
        F: FnMut(&Chunk<'a, 'b>),
    {
        recursive_visit(self.input, &self.token, f);
    }
}
