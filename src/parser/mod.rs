use std::vec::IntoIter;

use anyhow::{anyhow, Error, Result};

use crate::utils::{peek_nth, Either, PeekNth};
pub use crate::{
    parser::ast::{Ast, Atom, Block, BranchType, Expr, IfBranch, Stmt, Visitable, Visitor},
    parser::token::{Affix, Token},
};

mod ast;
mod token;

macro_rules! expect {
    ($self: ident, $token: expr) => {{
        if !$self.peek_eq($token) {
            return Err(anyhow!("Expected {}!", $token));
        } else {
            $self.next_token();
        }
    }};
}

pub struct Parser {
    errors: Vec<Error>,
    tokens: PeekNth<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            errors: Vec::new(),
            tokens: peek_nth(tokens.into_iter()),
        }
    }

    // UTILITIES

    pub fn parse(mut self) -> Result<Ast, Vec<Error>> {
        let mut stmts: Vec<Stmt> = vec![];
        while self.tokens.peek().is_some() {
            match self.stmt() {
                Ok(stmt) => {
                    stmts.push(stmt);
                }
                Err(e) => {
                    self.errors.push(e);

                    while let Some(token) = self.tokens.peek() {
                        if !token.is_stmt() {
                            self.next_token();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if self.errors.is_empty() {
            // Global block wrapping all statements
            Ok(Ast(stmts))
        } else {
            Err(self.errors)
        }
    }

    fn peek_bp(&mut self, affix: Affix) -> usize {
        self.tokens.peek().map_or(0, |t| t.bp(affix))
    }

    fn peek_eq(&mut self, expected: Token) -> bool {
        self.tokens.peek().map_or(false, |t| t == &expected)
    }

    fn peek_token(&mut self) -> &Token {
        self.tokens.peek().expect("Tried to peek empty iterator")
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().expect("Tried to iterate empty iterator")
    }

    // GRAMMAR

    fn parse_block(&mut self) -> Result<Block> {
        expect!(self, Token::OpenBrace);

        let mut block_items: Vec<Either<Stmt, Expr>> = Vec::new();

        while !self.peek_eq(Token::CloseBrace) {
            block_items.push(self.parse_expr_or_stmt()?);
        }

        // TODO: dirty WIP logic
        let final_expr = if block_items.last().map(|i| i.is_right()).unwrap_or(false) {
            block_items.pop().unwrap().into_right().ok().map(Box::new)
        } else {
            None
        };

        let body = block_items
            .into_iter()
            .map(|i| {
                i.into_left().map_err(|_| {
                    anyhow!("Standalone expressions in block are only supported at the end!")
                })
            })
            .collect::<Result<Vec<Stmt>>>()?;

        expect!(self, Token::CloseBrace);

        Ok(Block { body, final_expr })
    }

    fn parse_if_branch(&mut self, branch_type: BranchType) -> Result<IfBranch> {
        expect!(self, Token::from(branch_type));
        let condition = match branch_type {
            BranchType::Else => Expr::Atom(Atom::Bool(true)),
            _ => self.expr(0)?,
        };

        let body = self.parse_block()?;

        Ok(IfBranch {
            branch_type,
            condition,
            body,
        })
    }

    fn parse_expr_or_stmt(&mut self) -> Result<Either<Stmt, Expr>> {
        match self.peek_token().is_stmt() {
            true => Ok(Either::Left(self.stmt()?)),
            false => {
                let expr = self.expr(0)?;
                if self.peek_eq(Token::Semicolon) {
                    expect!(self, Token::Semicolon);
                    Ok(Either::Left(Stmt::Expr { expr }))
                } else {
                    Ok(Either::Right(expr))
                }
            }
        }
    }

    // EXPRESSIONS

    fn expr(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr = self.prefix()?;
        while self.peek_bp(Affix::Infix) > rbp {
            expr = self.binary_expr(expr)?;
        }
        Ok(expr)
    }

    fn prefix(&mut self) -> Result<Expr> {
        let token = self.peek_token();
        match token {
            Token::Minus => self.unary_expr(),
            Token::Bang => self.unary_expr(),
            Token::Identifier(_) => self.var_expr(),
            Token::OpenParenthesis => self.grouping_expr(),
            Token::OpenBrace => self.block_expr(),
            Token::If => self.if_expr(),
            Token::While => self.while_expr(),
            _ => self.atom_expr(),
        }
    }

    fn var_expr(&mut self) -> Result<Expr> {
        let token = self.next_token();
        if let Ok(identifier) = token.into_identifier() {
            // If next token is an assignment, then we are parsing binary expression
            // In order to assign some value to variable in VM we're gonna need this to
            // evaluate to variable's reference, not its value.
            let is_ref = self.peek_eq(Token::Assign);
            Ok(Expr::Var { identifier, is_ref })
        } else {
            Err(anyhow!("Invalid token got into var_expr call!"))
        }
    }

    fn atom_expr(&mut self) -> Result<Expr> {
        let token = self.next_token();
        match token {
            Token::Text(text) => Ok(Expr::Atom(Atom::Text(text))),
            Token::Number(num) => Ok(Expr::Atom(Atom::Number(num))),
            Token::False => Ok(Expr::Atom(Atom::Bool(false))),
            Token::True => Ok(Expr::Atom(Atom::Bool(true))),
            Token::Null => Ok(Expr::Atom(Atom::Null)),
            _ => Err(anyhow!(
                "This token is not supported by the parser: {}",
                token
            )),
        }
    }

    fn grouping_expr(&mut self) -> Result<Expr> {
        let open_paren = self.next_token();
        let bp = open_paren.bp(Affix::Prefix);
        let expr = self.expr(bp)?;

        expect!(self, Token::CloseParenthesis);

        Ok(Expr::Grouping {
            expr: Box::new(expr),
        })
    }

    fn unary_expr(&mut self) -> Result<Expr> {
        let operator = self.next_token();
        let bp = operator.bp(Affix::Prefix);
        let expr = self.expr(bp)?;

        Ok(Expr::Unary {
            expr: Box::new(expr),
            operator,
        })
    }

    fn binary_expr(&mut self, left: Expr) -> Result<Expr> {
        let operator = self.next_token();
        let right = self.expr(operator.bp(Affix::Infix))?;

        Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn block_expr(&mut self) -> Result<Expr> {
        Ok(Expr::Block {
            body: self.parse_block()?,
        })
    }

    fn if_expr(&mut self) -> Result<Expr> {
        let mut branches: Vec<IfBranch> = vec![];

        branches.push(self.parse_if_branch(BranchType::If)?);

        while self.peek_eq(Token::ElseIf) {
            branches.push(self.parse_if_branch(BranchType::ElseIf)?);
        }

        if self.peek_eq(Token::Else) {
            branches.push(self.parse_if_branch(BranchType::Else)?);
        }

        Ok(Expr::If { branches })
    }

    fn while_expr(&mut self) -> Result<Expr> {
        expect!(self, Token::While);
        let condition = Box::new(self.expr(0)?);
        let body = self.parse_block()?;

        Ok(Expr::While { condition, body })
    }

    // STATEMENTS

    fn stmt(&mut self) -> Result<Stmt> {
        match self.peek_token() {
            Token::Print => self.print_stmt(),
            Token::Var => self.var_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        let _token = self.next_token();
        let expr = self.expr(0)?;
        expect!(self, Token::Semicolon);
        Ok(Stmt::Print { expr })
    }

    fn expr_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expr(0)?;
        expect!(self, Token::Semicolon);
        Ok(Stmt::Expr { expr })
    }

    fn var_stmt(&mut self) -> Result<Stmt> {
        let _token = self.next_token();
        if let Ok(identifier) = self.next_token().into_identifier() {
            expect!(self, Token::Assign);
            let expr = self.expr(0)?;
            expect!(self, Token::Semicolon);
            Ok(Stmt::Var { expr, identifier })
        } else {
            Err(anyhow!("Something went wrong"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // EXPRESSIONS
    // TODO: add tests for the error detection, error messages, struct helpers
    #[test]
    fn number_atom() {
        let mut parser = Parser::new(vec![Token::Number(60.0)]);
        assert_eq!(parser.expr(0).unwrap(), Expr::Atom(Atom::Number(60.0)));
    }

    #[test]
    fn string_atom() {
        let mut parser = Parser::new(vec![Token::Text(String::from("hello"))]);
        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Atom(Atom::Text(String::from("hello")))
        );
    }

    #[test]
    fn grouping_expr() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Grouping {
                expr: Box::new(Expr::Atom(Atom::Number(10.0)))
            }
        )
    }

    #[test]
    fn unary_expr() {
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Unary {
                operator: Token::Minus,
                expr: Box::new(Expr::Grouping {
                    expr: Box::new(Expr::Atom(Atom::Number(10.0)))
                }),
            }
        )
    }

    #[test]
    fn binary_expr() {
        let mut parser = Parser::new(vec![Token::Number(10.0), Token::Plus, Token::Number(20.0)]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Token::Plus,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            }
        )
    }

    /// Assignment is just a simple binary operation
    #[test]
    fn binary_assignment_expr() {
        let mut parser = Parser::new(vec![
            Token::Identifier(String::from("var")),
            Token::Assign,
            Token::Number(20.0),
        ]);

        assert_eq!(
            parser.expr(0).expect("Couldn't parse expression"),
            Expr::Binary {
                left: Box::new(Expr::Var {
                    identifier: String::from("var"),
                    is_ref: true,
                }),
                operator: Token::Assign,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            }
        )
    }

    #[test]
    fn complicated_binary_expr() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(-1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::CloseParenthesis,
            Token::Star,
            Token::Number(3.0),
            Token::Minus,
            Token::Number(-4.0),
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Grouping {
                        expr: Box::new(Expr::Binary {
                            left: Box::new(Expr::Atom(Atom::Number(-1.0))),
                            operator: Token::Plus,
                            right: Box::new(Expr::Atom(Atom::Number(2.0))),
                        })
                    }),
                    operator: Token::Star,
                    right: Box::new(Expr::Atom(Atom::Number(3.0))),
                },),
                operator: Token::Minus,
                right: Box::new(Expr::Atom(Atom::Number(-4.0))),
            }
        )
    }

    /// Parser uses Patt Parsing to determine the binding power of infix/prefix/postfix operators
    /// so they are parsed in the correct order.
    /// E.g 2 + 8 * 10 is parsed as Binary<2 + Binary<8 * 10>>, instead of Binary<10 * Binary<2 +8>>
    #[test]
    fn handle_binding_power() {
        let mut parser = Parser::new(vec![
            Token::Number(2.0),
            Token::Plus,
            Token::Number(8.0),
            Token::Star,
            Token::Number(10.0),
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(2.0))),
                operator: Token::Plus,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Atom(Atom::Number(8.0))),
                    operator: Token::Star,
                    right: Box::new(Expr::Atom(Atom::Number(10.0))),
                }),
            }
        )
    }

    #[test]
    fn var_expr() {
        let mut parser = Parser::new(vec![Token::Identifier(String::from("variable"))]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Var {
                is_ref: false,
                identifier: String::from("variable"),
            }
        )
    }

    #[test]
    fn block_expr() {
        let mut parser = Parser::new(vec![
            Token::OpenBrace,
            Token::Var,
            Token::Identifier(String::from("var")),
            Token::Assign,
            Token::Number(10.0),
            Token::Semicolon,
            Token::CloseBrace,
        ]);

        assert_eq!(
            parser
                .block_expr()
                .expect("Failed to parse block expression"),
            Expr::Block {
                body: Block {
                    body: vec![Stmt::Var {
                        identifier: String::from("var"),
                        expr: Expr::Atom(Atom::Number(10.0)),
                    }],
                    final_expr: None,
                }
            }
        )
    }

    // STATEMENTS

    #[test]
    fn print_stmt() {
        assert_eq!(
            Parser::new(vec![
                Token::Print,
                Token::Identifier(String::from("variable")),
                Token::Semicolon,
            ])
            .stmt()
            .unwrap(),
            Stmt::Print {
                expr: Expr::Var {
                    is_ref: false,
                    identifier: String::from("variable"),
                }
            }
        );
        assert_eq!(
            Parser::new(vec![
                Token::Print,
                Token::Number(10.0),
                Token::Plus,
                Token::Number(20.0),
                Token::Semicolon,
            ])
            .stmt()
            .unwrap(),
            Stmt::Print {
                expr: Expr::Binary {
                    left: Box::new(Expr::Atom(Atom::Number(10.0))),
                    operator: Token::Plus,
                    right: Box::new(Expr::Atom(Atom::Number(20.0))),
                }
            }
        )
    }

    #[test]
    fn var_stmt() {
        assert_eq!(
            Parser::new(vec![
                Token::Var,
                Token::Identifier(String::from("variable")),
                Token::Assign,
                Token::Number(10.0),
                Token::Semicolon,
            ])
            .stmt()
            .unwrap(),
            Stmt::Var {
                identifier: String::from("variable"),
                expr: Expr::Atom(Atom::Number(10.0)),
            }
        );
    }
}
