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

    let mut parser = Parser::new(lexer);
    let ast = parser.parse_program()?;

    println!("{:#?}", ast);

    Ok(())
}
