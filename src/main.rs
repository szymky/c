use std::fs;

use anyhow::Result;
use logos::Logos;

use crate::parser::Parser;

mod ast;
mod lexer;
mod parser;

fn main() -> Result<()> {
    let source = fs::read_to_string("test.c")?;

    let lexer = lexer::Token::lexer(&source);

    for (token, span) in lexer.clone().spanned() {
        match token {
            Ok(t) => {
                let begin = span.start;
                let end = span.end;
                println!("{}..{}, {:?}, {}", begin, end, t, &source[begin..end]);
            }
            Err(_) => panic!(),
        }
    }

    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program()?;

    println!("{:#?}", ast);

    Ok(())
}
