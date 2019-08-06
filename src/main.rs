mod lexer;
mod util;
mod shell;

use std::fmt;
use std::io;
use std::io::prelude::*;
use shell::Value;
use streaming_iterator::StreamingIterator;

fn main() -> Result<(), io::Error> {
//    let mut stdin = io::stdin();
    let mut stdin = std::fs::File::open("/nix/store/9g6g97m31n5ns47fpcri70397mx5cz3y-python2.7-botocore-1.12.96/lib/python2.7/site-packages/botocore/data/ec2/2016-11-15/service-2.json")?;
    let mut s = String::new();
    stdin.read_to_string(&mut s)?;
    let l = lexer::Lexer::new(&s);
    let mut p = shell::Parser::new(l);

    let mut stdout = io::BufWriter::new(io::stdout());
//    let mut stdout = io::sink();
    while let Some((namespace, value)) = p.next() {
        stdout.write_all(namespace.as_bytes()).expect("couldn't write all namespace");
        stdout.write_all("=\"".as_bytes());
        stdout.write_all(match value {
            Value::Text(str) => str.as_bytes(),
            Value::Number(num) => "bruhnumber".as_bytes(),
            Value::Null => "null".as_bytes(),
            Value::True => "true".as_bytes(),
            Value::False => "false".as_bytes(),
        });
        stdout.write_all("\"\n".as_bytes());
    }

    Ok(())
}
