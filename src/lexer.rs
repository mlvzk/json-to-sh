use std::iter::Peekable;
use std::str::Chars;

#[derive(PartialEq, Debug)]
pub enum Value {
    True,         // true
    False,        // false
    Null,         // null
    Text(String), // "str"
    Number(f64),  // 1.23e01
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Illegal(String),
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]
    Colon,    // :
    Comma,    // ,
    Value(Value),
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! progress {
            ($x:expr) => {{
                self.chars.nth(0);
                Some($x)
            }};
            ($x:expr, $y:expr) => {{
                self.chars.nth($y);
                Some($x)
            }};
        }

        match self.chars.peek()? {
            ' ' | '\t' | '\n' => progress!(self.next()?),
            '[' => progress!(Token::LBracket),
            ']' => progress!(Token::RBracket),
            '{' => progress!(Token::LBrace),
            '}' => progress!(Token::RBrace),
            ':' => progress!(Token::Colon),
            ',' => progress!(Token::Comma),
            't' => progress!(Token::Value(Value::True), 3),
            'f' => progress!(Token::Value(Value::False), 4),
            'n' => progress!(Token::Value(Value::Null), 3),
            '"' => {
                self.chars.next();
                let text = eat_string(&mut self.chars)?;
                Some(Token::Value(Value::Text(text)))
            }
            '-' | '0'...'9' => {
                let num = eat_number(&mut self.chars)?;
                Some(Token::Value(Value::Number(num)))
            }
            c => {
                let s = c.to_string();
                progress!(Token::Illegal(s))
            }
        }
    }
}

fn eat_string(chars: &mut Peekable<Chars>) -> Option<String> {
    let mut s = String::with_capacity(15);

    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(s.to_owned()),
            '\\' => match chars.next()? {
                '\\' => s.push('\\'),
                '/' => s.push('/'),
                '"' => s.push('"'),
                'n' => s.push('\n'),
                't' => s.push('\t'),
                'b' => s.push(8 as char),
                'f' => s.push(12 as char),
                'r' => s.push(13 as char),
                'u' => {
                    if let Ok(num) = u8::from_str_radix(&chars.take(4).collect::<String>(), 16) {
                        s.push(num as char);
                    }
                }
                _ => {}
            },
            _ => s.push(c),
        }
    }

    Some(s.to_owned())
}

fn eat_number(chars: &mut Peekable<Chars>) -> Option<f64> {
    let mut s = String::with_capacity(10);
    s.push(chars.next()?);

    while let Some(&c) = chars.peek() {
        match c {
            '0'...'9' => s.push(c),
            '.' => s.push(c),
            'e' => s.push(c),
            '-' | '+' => s.push(c),
            _ => break,
        }

        chars.next();
    }

    match s.parse::<f64>() {
        Ok(num) => Some(num),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_tokens {
        ($i:expr, $o:expr) => {{
            println!("Lexer output: {:?}", Lexer::new($i).collect::<Vec<Token>>());
            assert!(Lexer::new($i).eq($o));
        }};
    }

    #[test]
    fn lexer_ignore_whitespace() {
        let input = " \t   \n   ";
        let output: Vec<Token> = vec![];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_comma() {
        let input = ",,";
        let output: Vec<Token> = vec![Token::Comma, Token::Comma];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_colon() {
        let input = "::";
        let output: Vec<Token> = vec![Token::Colon, Token::Colon];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_text() {
        let input = r#""test","123""#;
        let output = vec![
            Token::Value(Value::Text("test".to_owned())),
            Token::Comma,
            Token::Value(Value::Text("123".to_owned())),
        ];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_number() {
        let input = "-01.23e-2,123";
        let output = vec![
            Token::Value(Value::Number(-1.23e-2_f64)),
            Token::Comma,
            Token::Value(Value::Number(123_f64)),
        ];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_array() {
        let input = "[]";
        let output = vec![Token::LBracket, Token::RBracket];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_object() {
        let input = "{}";
        let output = vec![Token::LBrace, Token::RBrace];

        assert_tokens!(input, output);
    }

    #[test]
    fn lexer_booleans() {
        let input = "true,false";
        let output = vec![
            Token::Value(Value::True),
            Token::Comma,
            Token::Value(Value::False),
        ];

        assert_tokens!(input, output);
    }
    #[test]
    fn lexer_null() {
        let input = "null,null";
        let output = vec![
            Token::Value(Value::Null),
            Token::Comma,
            Token::Value(Value::Null),
        ];

        assert_tokens!(input, output);
    }
}
