#![feature(exclusive_range_pattern)]
mod lexer;
mod parser;

use std::fmt;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
struct ShellVar {
    name: String,
    value: String,
}

impl ShellVar {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

impl std::fmt::Display for ShellVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let escaped_name = self
            .name
            .chars()
            .filter(|c| match c {
                '0'...'9' | 'a'...'z' | 'A'...'Z' | '_' => true,
                _ => false,
            })
            .collect::<String>();

        write!(f, r#"{}="{}""#, escaped_name, self.value)
    }
}

trait ToShellVar {
    fn to_shell_var(&self, namespace: &str) -> Vec<ShellVar>;
}

impl ToShellVar for parser::Value {
    fn to_shell_var(&self, namespace: &str) -> Vec<ShellVar> {
        match self {
            parser::Value::Null => vec![ShellVar::new(String::from(namespace), "null".to_owned())],
            parser::Value::True => vec![ShellVar::new(String::from(namespace), "true".to_owned())],
            parser::Value::False => {
                vec![ShellVar::new(String::from(namespace), "false".to_owned())]
            }
            parser::Value::Text(text) => vec![ShellVar::new(String::from(namespace), text.clone())],
            parser::Value::Number(num) => {
                vec![ShellVar::new(String::from(namespace), num.to_string())]
            }
            parser::Value::Array(arr) => arr
                .iter()
                .enumerate()
                .map(|(i, v)| v.to_shell_var(&format!("{}_{}", namespace, i)))
                .flatten()
                .collect(),
            parser::Value::Object(obj) => obj
                .iter()
                .map(|(k, v)| v.to_shell_var(&format!("{}_{}", namespace, k)))
                .flatten()
                .collect(),
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut stdio = io::stdin();
    let mut s = String::new();
    stdio.read_to_string(&mut s)?;
    let l = lexer::Lexer::new(&s);
    let p = parser::Parser::new(l);

    p.map(|v| v.to_shell_var("root"))
        .flatten()
        .for_each(|sv| println!("{}", sv.to_string()));

    Ok(())
}
