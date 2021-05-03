use crate::parse::operator::UnaryOperator;
use crate::{
    common::error::ParseErrorCause,
    parse::{expr::atom::AtomicValue, operator::BinaryOperator, ParseResult, Parser, Spanned},
    token::Token,
};
use derive_more::Display;
use std::convert::TryInto;

pub(crate) mod atom;

#[derive(Debug, Display, Clone, PartialEq)]
pub(crate) enum Expr {
    Atom(Spanned<AtomicValue>),
    #[display(fmt = "({} {} {})", op, lhs, rhs)]
    Binary {
        lhs: Box<Expr>,
        op: Spanned<BinaryOperator>,
        rhs: Box<Expr>,
    },
    #[display(fmt = "({} {})", op, rhs)]
    Unary {
        op: Spanned<UnaryOperator>,
        rhs: Box<Expr>,
    },
}

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expression_bp(0)
    }

    pub(super) fn parse_expression_bp(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let mut lhs: Expr = match self.peek() {
            Token::Operator(op) => {
                let ((), r_bp) = op.prefix_bp();
                let op = self.construct_spanned(op.try_into()?)?;
                let rhs = Box::new(self.parse_expression_bp(r_bp)?);
                Expr::Unary { op, rhs }
            }
            _ => self.parse_atom()?,
        };

        loop {
            let operator = match self.peek() {
                Token::Operator(operator) => operator,
                Token::Eof => break,
                _ => return Err(ParseErrorCause::UnexpectedToken),
            };

            let (l_bp, r_bp) = operator.infix_bp();
            if l_bp < min_bp {
                break;
            }

            // Advance and construct spanned operator
            let op = {
                let lexeme = self.advance()?;
                Spanned {
                    val: operator.try_into()?,
                    span: lexeme.span(),
                }
            };

            let rhs = self.parse_expression_bp(r_bp)?;
            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expr(input: &str) -> Expr {
        let mut parser = Parser::new(input);
        parser.parse_expression().unwrap()
    }

    fn assert_expr(input: &str, expected: &str) {
        assert_eq!(expr(input).to_string(), expected)
    }

    #[test]
    fn parses_simple_binary_expression() {
        assert_expr("1 + 2", "(+ 1 2)");
        assert_expr("1 - 2", "(- 1 2)");
        assert_expr("1 * 2", "(* 1 2)");
        assert_expr("1 / 2", "(/ 1 2)");
        assert_expr("1 % 2", "(% 1 2)");
        assert_expr("1 ** 2", "(** 1 2)");
        assert_expr("1 == 2", "(== 1 2)");
        assert_expr("1 != 2", "(!= 1 2)");
        assert_expr("1 < 2", "(< 1 2)");
        assert_expr("1 <= 2", "(<= 1 2)");
        assert_expr("1 > 2", "(> 1 2)");
        assert_expr("1 >= 2", "(>= 1 2)");
        assert_expr("1 or 2", "(or 1 2)");
        assert_expr("1 and 2", "(and 1 2)");
    }

    #[test]
    fn parses_binary_expressions_with_equal_precedence() {
        // logical
        assert_expr("1 or 2 or 3", "(or (or 1 2) 3)");
        assert_expr("1 and 2 and 3", "(and (and 1 2) 3)");
        // comparison, this will get discarded during static analysis,
        // but we want to ensure that parser doesn't surprise us
        assert_expr("1 == 2 == 3", "(== (== 1 2) 3)");
        assert_expr("1 != 2 != 3", "(!= (!= 1 2) 3)");
        assert_expr("1 < 2 < 3", "(< (< 1 2) 3)");
        assert_expr("1 <= 2 <= 3", "(<= (<= 1 2) 3)");
        assert_expr("1 > 2 > 3", "(> (> 1 2) 3)");
        assert_expr("1 >= 2 >= 3", "(>= (>= 1 2) 3)");
        // addition and subtraction
        assert_expr("1 + 2 + 3", "(+ (+ 1 2) 3)");
        assert_expr("1 + 2 + 3 + 4", "(+ (+ (+ 1 2) 3) 4)");
        assert_expr("1 + 2 - 3", "(- (+ 1 2) 3)");
        assert_expr("1 - 2 + 3", "(+ (- 1 2) 3)");
        // multiplication, division, modulo
        assert_expr("1 * 2 * 3", "(* (* 1 2) 3)");
        assert_expr("1 / 2 * 3", "(* (/ 1 2) 3)");
        assert_expr("1 * 2 / 3", "(/ (* 1 2) 3)");
        assert_expr("1 % 2 % 3", "(% (% 1 2) 3)");
        assert_expr("1 * 2 / 3 % 4", "(% (/ (* 1 2) 3) 4)");
        // exponent
        assert_expr("1 ** 2 ** 3", "(** (** 1 2) 3)");
    }

    #[test]
    fn parses_binary_expressions_with_bigger_precedence() {
        // logical operators precedes comparison
        assert_expr("1 and 2 < 3", "(and 1 (< 2 3))");
        assert_expr("1 < 2 and 3", "(and (< 1 2) 3)");
        assert_expr("1 or 2 < 3", "(or 1 (< 2 3))");
        assert_expr("1 < 2 or 3", "(or (< 1 2) 3)");
        // comparison precedes addition and subtraction
        assert_expr("1 + 2 > 3", "(> (+ 1 2) 3)");
        assert_expr("1 > 2 + 3", "(> 1 (+ 2 3))");
        assert_expr("1 > 2 - 3", "(> 1 (- 2 3))");
        assert_expr("1 - 2 > 3", "(> (- 1 2) 3)");
        // addition and subtraction precedes multiplication, division and modulo
        assert_expr("1 + 2 * 3", "(+ 1 (* 2 3))");
        assert_expr("1 * 2 + 3", "(+ (* 1 2) 3)");
        assert_expr("1 - 2 / 3", "(- 1 (/ 2 3))");
        assert_expr("1 / 2 - 3", "(- (/ 1 2) 3)");
        assert_expr("1 + 2 % 3", "(+ 1 (% 2 3))");
        assert_expr("1 % 2 - 3", "(- (% 1 2) 3)");
        // multiplication, division and modulo precedes exponent
        assert_expr("1 * 2 ** 3", "(* 1 (** 2 3))");
        assert_expr("1 ** 2 / 3", "(/ (** 1 2) 3)");
        assert_expr("1 % 2 ** 3", "(% 1 (** 2 3))");
    }

    #[test]
    fn parses_unary_expressions() {
        assert_expr("- -1", "(- -1)");
        assert_expr("- 2 + 2", "(- (+ 2 2))");
        assert_expr("!true", "(! true)");
        assert_expr("!!true", "(! (! true))");
        assert_expr("!!!true", "(! (! (! true)))");
        assert_expr("!!!!true", "(! (! (! (! true))))");

        assert_expr("--5", "(- -5)");
        assert_expr("---5", "(- (- -5))");
        assert_expr("----5", "(- (- (- -5)))");
    }

    #[test]
    fn parses_combined_expression() {
        assert_expr("!true == false", "(== (! true) false)");
        assert_expr("!!true == !false", "(== (! (! true)) (! false))");
        assert_expr("2 >= 10 + 3", "(>= 2 (+ 10 3))");
        assert_expr("2 + 2 ** 3 >= 10 + 3", "(>= (+ 2 (** 2 3)) (+ 10 3))");
    }
}
