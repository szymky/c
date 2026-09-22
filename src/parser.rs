use anyhow::{Context, Result, bail};
use logos::Lexer;

use crate::ast::*;
use crate::lexer::Token;

pub struct Parser<'a> {
    lexer: Lexer<'a, Token>,
    current: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a, Token>) -> Self {
        let current = lexer.next().and_then(|r| r.ok());
        Parser { lexer, current }
    }

    fn skip_trivia(&mut self) {
        while matches!(
            self.current,
            Some(Token::Newline) | Some(Token::SingleLineComment) | Some(Token::BlockComment)
        ) {
            self.advance();
        }
    }

    fn advance(&mut self) {
        self.current = self.lexer.next().and_then(|r| r.ok());
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.current.as_ref() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            bail!(
                "Expected token {:?}, but found {:?}",
                expected,
                self.current
            );
        }
    }

    fn slice(&self) -> String {
        self.lexer.slice().to_string()
    }

    fn parse_type_suffix(&mut self, mut base_ty: Type) -> Result<Type> {
        self.skip_trivia();

        while self.current == Some(Token::OpenBracket) {
            self.advance();

            self.skip_trivia();

            let size_expr = if self.current != Some(Token::CloseBracket) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.expect(Token::CloseBracket)?;

            base_ty = Type::Array {
                element_type: Box::new(base_ty),
                size: size_expr,
            };
        }

        Ok(base_ty)
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = Vec::new();

        while self.current.is_some() {
            if matches!(
                self.current,
                Some(Token::Newline)
                    | Some(Token::Preprocessor)
                    | Some(Token::SingleLineComment)
                    | Some(Token::BlockComment)
            ) {
                self.advance();
                continue;
            }

            let decl = self
                .parse_declaration()
                .context("Failed to parse top-level declaration")?;
            declarations.push(decl);
        }

        Ok(Program { declarations })
    }

    pub fn parse_declaration(&mut self) -> Result<Declaration> {
        self.skip_trivia();

        if self.current == Some(Token::Struct) {
            self.advance();

            let struct_name = if let Some(Token::Identifier) = &self.current {
                let name = self.slice();
                self.advance();
                Some(name)
            } else {
                None
            };

            if self.current == Some(Token::OpenBrace) {
                self.advance();
                let mut fields = Vec::new();

                while self.current.is_some() && self.current != Some(Token::CloseBrace) {
                    self.skip_trivia();
                    if self.current == Some(Token::CloseBrace) {
                        break;
                    }
                    let mut field_ty = self
                        .parse_type()
                        .context("Failed to parse struct field type")?;

                    let field_name = match self.current.take() {
                        Some(Token::Identifier) => {
                            let s = self.slice();
                            self.advance();
                            s
                        }
                        other => bail!("Expected field name, found {:?}", other),
                    };

                    field_ty = self.parse_type_suffix(field_ty)?;
                    self.expect(Token::Semicolon)?;

                    fields.push(StructField {
                        ty: field_ty,
                        name: field_name,
                    });
                }

                self.expect(Token::CloseBrace)?;
                self.expect(Token::Semicolon)?;

                return Ok(Declaration::StructDef {
                    name: struct_name,
                    fields,
                });
            }

            let ty = Type::Struct(struct_name.context("Expected struct name or '{'")?);
            return self.parse_var_or_function_decl(ty);
        }

        let ty = self.parse_type().context("Expected type specifier")?;
        self.parse_var_or_function_decl(ty)
    }

    fn parse_var_or_function_decl(&mut self, mut ty: Type) -> Result<Declaration> {
        self.skip_trivia();

        let name = match self.current.take() {
            Some(Token::Identifier) => {
                let s = self.slice();
                self.advance();
                s
            }
            other => bail!("Expected identifier name, found {:?}", other),
        };

        if self.current == Some(Token::OpenParen) {
            let func = self.parse_function_body(ty, name)?;
            Ok(Declaration::Function(func))
        } else {
            ty = self.parse_type_suffix(ty)?;

            let init = if self.current == Some(Token::Assign) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect(Token::Semicolon)?;
            Ok(Declaration::GlobalVar {
                ty,
                name,
                initializer: init,
            })
        }
    }
    fn parse_unary_expression(&mut self) -> Result<Expr> {
        self.skip_trivia();

        let op = match &self.current {
            Some(Token::Increment) => Some(UnaryOp::PreInc),
            Some(Token::Decrement) => Some(UnaryOp::PreDec),
            Some(Token::BitAnd) => Some(UnaryOp::AddrOf),
            Some(Token::Star) => Some(UnaryOp::Deref),
            Some(Token::Minus) => Some(UnaryOp::Neg),
            Some(Token::LogicalNot) => Some(UnaryOp::LogicalNot),
            Some(Token::BitNot) => Some(UnaryOp::BitNot),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary_expression()?;
            Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
            })
        } else {
            self.parse_postfix_expression()
        }
    }

    fn parse_function_call_args(&mut self) -> Result<Vec<Expr>> {
        self.expect(Token::OpenParen)?;
        let mut args = Vec::new();

        if self.current != Some(Token::CloseParen) {
            loop {
                self.skip_trivia();

                let arg = self.parse_expression()?;
                args.push(arg);

                self.skip_trivia();

                if self.current == Some(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.expect(Token::CloseParen)?;
        Ok(args)
    }

    fn parse_function_body(&mut self, return_type: Type, name: String) -> Result<FunctionDecl> {
        self.expect(Token::OpenParen)?;

        let mut params = Vec::new();
        if self.current != Some(Token::CloseParen) {
            loop {
                self.skip_trivia();

                let mut p_type = self.parse_type()?;
                let p_name = if let Some(Token::Identifier) = &self.current {
                    let s = self.slice();
                    self.advance();
                    Some(s)
                } else {
                    None
                };

                p_type = self.parse_type_suffix(p_type)?;

                params.push(Param {
                    ty: p_type,
                    name: p_name,
                });

                if self.current == Some(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::CloseParen)?;

        self.skip_trivia();

        if self.current == Some(Token::Semicolon) {
            self.advance();
            Ok(FunctionDecl {
                return_type,
                name,
                params,
                body: None,
            })
        } else if self.current == Some(Token::OpenBrace) {
            let body = self.parse_block()?;
            Ok(FunctionDecl {
                return_type,
                name,
                params,
                body: Some(body),
            })
        } else {
            bail!(
                "Expected ';' or '{{' after function signature, found {:?}",
                self.current
            );
        }
    }

    pub fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        self.expect(Token::OpenBrace)?;
        let mut stmts = Vec::new();

        while self.current.is_some() && self.current != Some(Token::CloseBrace) {
            self.skip_trivia();
            if self.current == Some(Token::CloseBrace) {
                break;
            }
            stmts.push(self.parse_statement()?);
        }

        self.expect(Token::CloseBrace)?;
        Ok(stmts)
    }

    pub fn parse_statement(&mut self) -> Result<Stmt> {
        self.skip_trivia();

        match &self.current {
            Some(Token::Return) => {
                self.advance();
                let expr = if self.current != Some(Token::Semicolon) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Return(expr))
            }

            Some(Token::If) => {
                self.advance();
                self.expect(Token::OpenParen)?;
                let cond = self.parse_expression()?;
                self.expect(Token::CloseParen)?;

                let then_b = Box::new(self.parse_statement()?);
                let else_b = if self.current == Some(Token::Else) {
                    self.advance();
                    Some(Box::new(self.parse_statement()?))
                } else {
                    None
                };
                Ok(Stmt::If {
                    condition: cond,
                    then_branch: then_b,
                    else_branch: else_b,
                })
            }

            Some(Token::While) => {
                self.advance();
                self.expect(Token::OpenParen)?;
                let cond = self.parse_expression()?;
                self.expect(Token::CloseParen)?;

                let body = Box::new(self.parse_statement()?);
                Ok(Stmt::While {
                    condition: cond,
                    body,
                })
            }

            Some(Token::For) => {
                self.advance();
                self.expect(Token::OpenParen)?;

                let init = if self.current == Some(Token::Semicolon) {
                    self.advance();
                    None
                } else {
                    Some(Box::new(self.parse_statement()?))
                };

                let condition = if self.current != Some(Token::Semicolon) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;

                let post = if self.current != Some(Token::CloseParen) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::CloseParen)?;

                let body = Box::new(self.parse_statement()?);

                Ok(Stmt::For {
                    init,
                    condition,
                    post,
                    body,
                })
            }

            Some(Token::OpenBrace) => {
                let stmts = self.parse_block()?;
                Ok(Stmt::Block(stmts))
            }

            Some(Token::Int)
            | Some(Token::FloatKeyword)
            | Some(Token::CharKeyword)
            | Some(Token::Struct) => {
                let mut ty = self.parse_type()?;
                let name = match self.current.take() {
                    Some(Token::Identifier) => {
                        let s = self.slice();
                        self.advance();
                        s
                    }
                    other => bail!("Expected variable name, found {:?}", other),
                };

                ty = self.parse_type_suffix(ty)?;

                let init = if self.current == Some(Token::Assign) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semicolon)?;
                Ok(Stmt::VarDecl {
                    ty,
                    name,
                    initializer: init,
                })
            }

            _ => {
                let expr = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    pub fn parse_array_init(&mut self) -> Result<Expr> {
        self.expect(Token::OpenBrace)
            .context("Expected '{' to start initializer list")?;

        let mut elements = Vec::new();

        self.skip_trivia();

        while self.current.is_some() && self.current != Some(Token::CloseBrace) {
            self.skip_trivia();

            if self.current == Some(Token::CloseBrace) {
                break;
            }

            let element = self.parse_expression()?;
            elements.push(element);

            self.skip_trivia();

            if self.current == Some(Token::Comma) {
                self.advance();
            } else if self.current != Some(Token::CloseBrace) {
                bail!(
                    "Expected ',' or '}}' in initializer list, found {:?}",
                    self.current
                );
            }
        }

        self.expect(Token::CloseBrace)
            .context("Expected '}' at end of initializer list")?;

        Ok(Expr::ArrayInit(elements))
    }

    fn parse_type(&mut self) -> Result<Type> {
        self.skip_trivia();

        let mut base_type = match &self.current {
            Some(Token::Int) => {
                self.advance();
                Type::Int
            }
            Some(Token::CharKeyword) => {
                self.advance();
                Type::Char
            }
            Some(Token::FloatKeyword) => {
                self.advance();
                Type::Float
            }
            Some(Token::Void) => {
                self.advance();
                Type::Void
            }
            Some(Token::Struct) => {
                self.advance();
                match self.current.take() {
                    Some(Token::Identifier) => {
                        let name = self.slice();
                        self.advance();
                        Type::Struct(name)
                    }
                    other => bail!(
                        "Expected identifier after struct keyword, found {:?}",
                        other
                    ),
                }
            }
            other => bail!("Invalid type specifier: {:?}", other),
        };

        while self.current == Some(Token::Star) {
            self.advance();
            base_type = Type::Pointer(Box::new(base_type));
        }

        Ok(base_type)
    }

    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_assignment()
            .context("Failed to parse expression")
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;

        while let Some(token) = &self.current {
            let op = match token {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let target = self.parse_equality()?;

        self.skip_trivia();

        let op = match &self.current {
            Some(Token::Assign) => Some(AssignmentOp::Assign),
            Some(Token::PlusAssign) => Some(AssignmentOp::AddAssign),
            Some(Token::MinusAssign) => Some(AssignmentOp::SubAssign),
            Some(Token::StarAssign) => Some(AssignmentOp::MulAssign),
            Some(Token::SlashAssign) => Some(AssignmentOp::DivAssign),
            Some(Token::PercentAssign) => Some(AssignmentOp::ModAssign),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let value = self.parse_expression()?;

            Ok(Expr::Assignment {
                op,
                target: Box::new(target),
                value: Box::new(value),
            })
        } else {
            Ok(target)
        }
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut left = self.parse_addition()?;

        while let Some(token) = &self.current {
            let op = match token {
                Token::LessThan => BinaryOp::LessThan,
                Token::GreaterThan => BinaryOp::GreaterThan,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplication()?;

        while let Some(token) = &self.current {
            let op = match token {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary_expression()?;

        while let Some(token) = &self.current {
            let op = match token {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary_expression()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_postfix_expression(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            self.skip_trivia();

            match &self.current {
                Some(Token::Increment) => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostInc,
                        operand: Box::new(expr),
                    };
                }

                Some(Token::Decrement) => {
                    self.advance();
                    expr = Expr::Unary {
                        op: UnaryOp::PostDec,
                        operand: Box::new(expr),
                    };
                }

                Some(Token::OpenBracket) => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(Token::CloseBracket)?;
                    expr = Expr::Index {
                        target: Box::new(expr),
                        index: Box::new(index),
                    };
                }

                Some(Token::Dot) => {
                    self.advance();
                    let field = match self.current.take() {
                        Some(Token::Identifier) => {
                            let s = self.slice();
                            self.advance();
                            s
                        }
                        other => bail!("Expected field name after '.', found {:?}", other),
                    };
                    expr = Expr::MemberAccess {
                        target: Box::new(expr),
                        field,
                    };
                }

                Some(Token::Arrow) => {
                    self.advance();
                    let field = match self.current.take() {
                        Some(Token::Identifier) => {
                            let s = self.slice();
                            self.advance();
                            s
                        }
                        other => bail!("Expected field name after '->', found {:?}", other),
                    };
                    expr = Expr::PointerMemberAccess {
                        target: Box::new(expr),
                        field,
                    };
                }

                Some(Token::OpenParen) => {
                    let args = self.parse_function_call_args()?;
                    expr = Expr::FunctionCall {
                        callee: Box::new(expr),
                        args,
                    };
                }

                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_trivia();

        let slice = self.lexer.slice().to_string();

        match &self.current {
            Some(Token::OpenBrace) => self.parse_array_init(),

            Some(Token::Integer) => {
                self.advance();
                let val = slice
                    .parse::<i64>()
                    .with_context(|| format!("Failed to parse integer '{slice}'"))?;
                Ok(Expr::IntLiteral(val))
            }

            Some(Token::FloatLiteral) => {
                self.advance();
                let clean_slice = slice.trim_end_matches(|c| c == 'f' || c == 'F');
                let val = clean_slice
                    .parse::<f64>()
                    .with_context(|| format!("Failed to parse float '{slice}'"))?;
                Ok(Expr::FloatLiteral(val))
            }

            Some(Token::StringLiteral) => {
                self.advance();
                Ok(Expr::StringLiteral(slice))
            }

            Some(Token::Identifier) => {
                self.advance();
                Ok(Expr::Variable(slice))
            }

            Some(Token::OpenParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::CloseParen)?;
                Ok(expr)
            }

            other => bail!("Unexpected token in primary expression: {:?}", other),
        }
    }
}
