use super::lexer;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
    Text(String),
    Number(f64),
    True,
    False,
    Null,
}

fn translate_value(token_value: lexer::Value) -> Value {
    match token_value {
        lexer::Value::True => Value::True,
        lexer::Value::False => Value::False,
        lexer::Value::Null => Value::Null,
        lexer::Value::Text(s) => Value::Text(s),
        lexer::Value::Number(num) => Value::Number(num),
    }
}

pub struct Parser<T>
where
    T: Iterator<Item = lexer::Token>,
{
    tokens: T,
}

impl<T> Parser<T>
where
    T: Iterator<Item = lexer::Token>,
{
    pub fn new(iter: T) -> Self {
        Parser { tokens: iter }
    }

    fn handle_token(&mut self, token: lexer::Token) -> Option<Value> {
        match token {
            lexer::Token::LBracket => Some(Value::Array(self.eat_array()?)),
            lexer::Token::LBrace => Some(Value::Object(self.eat_object()?)),
            lexer::Token::Value(val) => Some(translate_value(val)),
            o => {
                eprintln!("unexpected token: {:?}", o);
                unimplemented!();
            }
        }
    }

    fn eat_object(&mut self) -> Option<Vec<(String, Value)>> {
        let mut v: Vec<(String, Value)> = vec![];

        while let Some(token) = self.tokens.next() {
            match token {
                lexer::Token::RBrace => break,
                lexer::Token::Comma => continue,
                lexer::Token::Value(lexer::Value::Text(text)) => {
                    if let Some(lexer::Token::Colon) = self.tokens.next() {
                        v.push((text, self.next()?));
                    } else {
                        unimplemented!();
                    }
                }
                _ => unimplemented!(),
            }
        }

        Some(v)
    }

    fn eat_array(&mut self) -> Option<Vec<Value>> {
        let mut v: Vec<Value> = vec![];

        while let Some(token) = self.tokens.next() {
            match token {
                lexer::Token::RBracket => break,
                lexer::Token::Comma => continue,
                other => v.push(self.handle_token(other)?),
            }
        }

        Some(v.to_vec())
    }
}

impl<T> Iterator for Parser<T>
where
    T: Iterator<Item = lexer::Token>,
{
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.tokens.next()?;
        Some(self.handle_token(token)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexer::Token;

    macro_rules! assert_parser {
        ($i:expr, $o:expr) => {{
            let parsed: Vec<Value> = Parser::new($i.into_iter()).collect();
            println!("Parser output: {:?}", parsed);
            assert!(parsed.into_iter().eq($o.into_iter()));
        }};
    }

    #[test]
    fn parser_array() {
        let input = vec![
            Token::LBracket,
            Token::LBracket,
            Token::RBracket,
            Token::RBracket,
        ];
        let output = vec![Value::Array(vec![Value::Array(vec![])])];

        assert_parser!(input, output);
    }

    #[test]
    fn parser_object() {
        let input = vec![
            Token::LBrace,
            Token::Value(lexer::Value::Text("key1".to_owned())),
            Token::Colon,
            Token::Value(lexer::Value::True),
            Token::Comma,
            Token::Value(lexer::Value::Text("key2".to_owned())),
            Token::Colon,
            Token::LBrace,
            Token::RBrace,
            Token::RBrace,
        ];

        let mut v = vec![];
        v.push(("key1".to_owned(), Value::True));
        v.push(("key2".to_owned(), Value::Object(vec![])));
        let expected = vec![Value::Object(v)];

        assert_parser!(input, expected);
    }
}
