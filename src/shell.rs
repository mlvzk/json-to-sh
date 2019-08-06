extern crate streaming_iterator;

use super::lexer::Token;
use super::lexer;
use streaming_iterator::StreamingIterator;

#[derive(Debug, PartialEq)]
pub enum Value {
    Text(String),
    Number(f64),
    True,
    False,
    Null,
}

pub enum State {
    Array(i32, usize),
    Object(usize),
    ExpectObjectValue,
}

pub struct Parser<T> {
    lexer: T,
    state: Vec<State>,
    curr: (String, Value),
    ended: bool,
}

impl<T> Parser<T> where T: Iterator<Item=Token> {
    pub fn new(lexer: T) -> Self {
        Self {
            lexer,
            state: Vec::new(),
            curr: ("root".to_string(), Value::Null),
            ended: false,
        }
    }
}

impl<T> StreamingIterator for Parser<T> where T: Iterator<Item=Token> {
    type Item = (String, Value);

    fn advance(&mut self) {
        if self.ended {
            return;
        }

        if let Some(token) = self.lexer.next() {
            if let Some(State::Array(index, _)) = self.state.last() {
                self.curr.0.push_str(&index.to_string());
            }

            match token {
                Token::LBracket => {
                    self.state.push(State::Array(0, self.curr.0.len()));
                    self.curr.0.push('_');
                    self.advance();
                }
                Token::RBracket => {
                    if let State::Array(_, trunc_len) = self.state.pop().expect("syntax error on ']") {
                        self.curr.0.truncate(trunc_len);
                        self.state.pop();
                    } else {
                        panic!("syntax error on ']'");
                    }
                    self.advance();
                }
                Token::LBrace => {
                    if let Some(State::ExpectObjectValue) = self.state.last() {
                        self.state.pop();
                    }
                    self.state.push(State::Object(self.curr.0.len()));
                    self.curr.0.push('_');
                    self.advance();
                }
                Token::RBrace => {
                    if let State::Object(trunc_len) = self.state.pop().expect("syntax error on '}") {
                        self.curr.0.truncate(trunc_len);
                    } else {
                        panic!("syntax error on '}'");
                    }
                    self.advance();
                }
                Token::Comma => {
                    match self.state.last_mut().expect("found comma, but no state") {
                        State::Array(ref mut i, trunc_len) => {
                            *i += 1;
                            self.curr.0.truncate(*trunc_len + 1);
                        }
                        State::Object(trunc_len) => {
                            self.curr.0.truncate(*trunc_len + 1);
                        }
                        State::ExpectObjectValue => panic!("expected value to object key, not comma"),
                    }
                    self.advance();
                }
                Token::Colon => {
                    self.advance();
                }
                Token::Value(value) => {
                    match value {
                        lexer::Value::Null => self.curr.1 = Value::Null,
                        lexer::Value::True => self.curr.1 = Value::True,
                        lexer::Value::False => self.curr.1 = Value::False,
                        lexer::Value::Number(n) => self.curr.1 = Value::Number(n),
                        lexer::Value::Text(txt) => match self.state.last() {
                            Some(State::Object(trunc_len)) => {
                                self.curr.0.truncate(*trunc_len + 1);
                                self.curr.0.push_str(&txt);
                                self.state.push(State::ExpectObjectValue);
                                self.advance();
                            }
                            _ => self.curr.1 = Value::Text(txt),
                        }
                    }
                    if let Some(State::ExpectObjectValue) = self.state.last() {
                        self.state.pop();
                    }
                }
                _ => unimplemented!(),
            }
        } else {
            self.ended = true;
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.ended {
            None
        } else {
            Some(&self.curr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn assert_parser(input: Vec<lexer::Token>, expectedOutput: Vec<(String, Value)>) {
        let mut it = Parser::new(input.into_iter());
        let mut i = 0;
        while let Some(o) = it.next() {
            assert_eq!(o, &expectedOutput[i], "; Index: {}", i.to_string());
            i += 1;
        }
    }

    #[test]
    fn array_of_primitives() {
        let input = vec![
            Token::LBracket,
            Token::Value(lexer::Value::True),
            Token::Comma,
            Token::Value(lexer::Value::False),
            Token::Comma,
            Token::Value(lexer::Value::Null),
            Token::Comma,
            Token::Value(lexer::Value::Number(23.)),
            Token::Comma,
            Token::Value(lexer::Value::Text("abc".to_string())),
            Token::RBracket,
        ];

        let expected = vec![
            ("root_0".to_string(), Value::True),
            ("root_1".to_string(), Value::False),
            ("root_2".to_string(), Value::Null),
            ("root_3".to_string(), Value::Number(23.)),
            ("root_4".to_string(), Value::Text("abc".to_string())),
        ];

        assert_parser(input, expected);
    }

    #[test]
    fn object_of_primitives() {
        let input = vec![
            Token::LBrace,
            Token::Value(lexer::Value::Text("a".to_string())),
            Token::Colon,
            Token::Value(lexer::Value::True),
            Token::Comma,
            Token::Value(lexer::Value::Text("b".to_string())),
            Token::Colon,
            Token::Value(lexer::Value::False),
            Token::Comma,
            Token::Value(lexer::Value::Text("c".to_string())),
            Token::Colon,
            Token::Value(lexer::Value::Text("test".to_string())),
            Token::RBrace,
        ];

        let expected = vec![
            ("root_a".to_string(), Value::True),
            ("root_b".to_string(), Value::False),
            ("root_c".to_string(), Value::Text("test".to_string())),
        ];

        assert_parser(input, expected);
    }

    #[test]
    fn array_of_objects() {
        let input = vec![
            Token::LBracket,
            Token::LBrace,
            Token::Value(lexer::Value::Text("a1".to_string())),
            Token::Colon,
            Token::Value(lexer::Value::True),
            Token::RBrace,
            Token::Comma,
            Token::LBrace,
            Token::Value(lexer::Value::Text("a2".to_string())),
            Token::Colon,
            Token::Value(lexer::Value::False),
            Token::RBrace,
            Token::RBracket,
        ];

        let expected = vec![
            ("root_0_a1".to_string(), Value::True),
            ("root_1_a2".to_string(), Value::False),
        ];

        assert_parser(input, expected);
    }

    #[test]
    fn object_of_array() {
        let input = vec![
            Token::LBrace,
            Token::Value(lexer::Value::Text("a".to_string())),
            Token::Colon,
            Token::LBracket,
            Token::Value(lexer::Value::Null),
            Token::Comma,
            Token::Value(lexer::Value::Text("v".to_string())),
            Token::RBracket,
            Token::RBrace,
        ];

        let expected = vec![
            ("root_a_0".to_string(), Value::Null),
            ("root_a_1".to_string(), Value::Text("v".to_string())),
        ];

        assert_parser(input, expected);
    }

    #[test]
    fn deep() {
        let input = vec![
            Token::LBrace,
            Token::Value(lexer::Value::Text("a".to_string())),
            Token::Colon,
            Token::LBracket,
            Token::LBrace,
            Token::Value(lexer::Value::Text("b".to_string())),
            Token::Colon,
            Token::LBracket,
            Token::Value(lexer::Value::True),
            Token::RBracket,
            Token::RBrace,
            Token::RBracket,
            Token::LBrace,
        ];

        let expected = vec![
            ("root_a_0_b_0".to_string(), Value::True),
        ];

        assert_parser(input, expected);
    }

    #[test]
    fn bruh() {
        let input = Lexer::new(r#"
{
  "version":"2.0",
  "metadata":{
    "apiVersion":"2016-11-15",
    "endpointPrefix":"ec2",
    "protocol":"ec2",
    "serviceAbbreviation":"Amazon EC2",
    "serviceFullName":"Amazon Elastic Compute Cloud",
    "serviceId":"EC2",
    "signatureVersion":"v4",
    "uid":"ec2-2016-11-15",
    "xmlNamespace":"http://ec2.amazonaws.com/doc/2016-11-15"
  },
  "operations":{
    "AcceptReservedInstancesExchangeQuote":{
      "name":"AcceptReservedInstancesExchangeQuote"
    }
  }
}
        "#);

        let expected = vec![
            ("root_version".to_string(), Value::Text("2.0".to_string())),
            ("root_metadata_apiVersion".to_string(), Value::Text("2016-11-15".to_string())),
            ("root_metadata_endpointPrefix".to_string(), Value::Text("ec2".to_string())),
            ("root_metadata_protocol".to_string(), Value::Text("ec2".to_string())),
            ("root_metadata_serviceAbbreviation".to_string(), Value::Text("Amazon EC2".to_string())),
            ("root_metadata_serviceFullName".to_string(), Value::Text("Amazon Elastic Compute Cloud".to_string())),
            ("root_metadata_serviceId".to_string(), Value::Text("EC2".to_string())),
            ("root_metadata_signatureVersion".to_string(), Value::Text("v4".to_string())),
            ("root_metadata_uid".to_string(), Value::Text("ec2-2016-11-15".to_string())),
            ("root_metadata_xmlNamespace".to_string(), Value::Text("http://ec2.amazonaws.com/doc/2016-11-15".to_string())),
            ("root_operations_AcceptReservedInstancesExchangeQuote_name".to_string(), Value::Text("AcceptReservedInstancesExchangeQuote".to_string())),
        ];

        assert_parser(input.collect(), expected);
    }
}
