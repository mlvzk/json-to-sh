mod lexer;
mod parser;

use std::fmt;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
struct ShellVar<'a> {
    name: &'a str,
    value: &'a str,
}

impl<'a> ShellVar<'a> {
    pub fn new(name: &'a str, value: &'a str) -> Self {
        Self { name, value }
    }
}

impl fmt::Display for ShellVar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let escaped_name = self
            .name
            .chars()
            .filter(|c| match c {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => true,
                _ => false,
            })
            .collect::<String>();

        write!(f, r#"{}="{}""#, escaped_name, self.value)
    }
}

impl parser::Value {
    fn traverse<F>(self, namespace: String, f: &mut F)
        where F: FnMut(ShellVar) -> (),
    {
        use parser::Value;
        match self {
            Value::Text(text) => f(ShellVar::new(&namespace, &text)),
            Value::Number(n) => f(ShellVar::new(&namespace, &n.to_string())),
            Value::True => f(ShellVar::new(&namespace, "true")),
            Value::False => f(ShellVar::new(&namespace, "false")),
            Value::Null => f(ShellVar::new(&namespace, "null")),
            Value::Array(arr) => {
                for (i, v) in arr.into_iter().enumerate() {
                    v.traverse(format!("{}_{}", namespace, i), f);
                }
            }
            Value::Object(obj) => {
                for (key, value) in obj {
                    value.traverse(format!("{}_{}", namespace, key), f);
                }
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut stdin = io::stdin();
    let mut s = String::new();
    stdin.read_to_string(&mut s)?;
    let l = lexer::Lexer::new(&s);
    let p = parser::Parser::new(l);

    let mut stdout = io::stdout();
    for v in p {
        v.traverse("root".to_string(), &mut |var| {
            stdout.write_all(var.to_string().as_bytes()).unwrap();
        });
    }

    Ok(())
}
