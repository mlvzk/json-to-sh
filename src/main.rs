mod lexer;
mod parser;
mod util;

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

        write!(f, r#"{}="{}"{}"#, escaped_name, self.value, "\n")
    }
}

impl parser::Value {
    fn traverse(self, namespace: &mut String, f: &mut impl FnMut(ShellVar) -> ())
    {
        use parser::Value;
        match self {
            Value::Text(text) => f(ShellVar::new(&namespace, &text)),
            Value::Number(n) => f(ShellVar::new(&namespace, &n.to_string())),
            Value::True => f(ShellVar::new(&namespace, "true")),
            Value::False => f(ShellVar::new(&namespace, "false")),
            Value::Null => f(ShellVar::new(&namespace, "null")),
            Value::Array(arr) => {
                namespace.push('_');
                let n_len = namespace.len();
                for (i, v) in arr.into_iter().enumerate() {
                    namespace.push_str(util::usize_to_str(i));
                    v.traverse(namespace, f);
                    namespace.truncate(n_len);
                }
            }
            Value::Object(obj) => {
                namespace.push('_');
                let n_len = namespace.len();
                for (key, value) in obj {
                    namespace.push_str(&key);
                    value.traverse(namespace, f);
                    namespace.truncate(n_len);
                }
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
//    let mut stdin = io::stdin();
    let mut stdin = std::fs::File::open("/nix/store/9g6g97m31n5ns47fpcri70397mx5cz3y-python2.7-botocore-1.12.96/lib/python2.7/site-packages/botocore/data/ec2/2016-11-15/service-2.json")?;
    let mut s = String::new();
    stdin.read_to_string(&mut s)?;
    let l = lexer::Lexer::new(&s);
    let p = parser::Parser::new(l);

    let mut stdout = io::BufWriter::new(io::stdout());
//    let mut stdout = io::sink();
    for v in p {
        let mut namespace = String::with_capacity(60);
        namespace.push_str("root");
        v.traverse(&mut namespace, &mut |var: ShellVar| {
            stdout.write_all(var.to_string().as_bytes()).unwrap();
        });
    }

    Ok(())
}
