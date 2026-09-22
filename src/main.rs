use std::fs;

use anyhow::Result;
use logos::Logos;

mod lexer;

fn main() -> Result<()> {
    let source = fs::read_to_string("test.c")?;

    let lexer = lexer::Token::lexer(&source);

    for (token, span) in lexer.spanned() {
        match token {
            Ok(t) => {
                let begin = span.start;
                let end = span.end;
                println!("{}..{}, {:?}, {}", begin, end, t, &source[begin..end]);
            }
            Err(_) => panic!(),
        }
    }

    Ok(())
}
