use crate::token::Token;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use logos::Span;

#[derive(Debug)]
pub(crate) struct ParseError {
    pub(crate) span: Span,
    pub(crate) cause: ParseErrorCause,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParseErrorCause {
    EndOfInput,
    UnexpectedToken,
    Expected(Token<'static>),
    ExpectedOneOf(Vec<Token<'static>>),
    ExpectedIdentifier,
    // Lexer
    TooMuchDots,
    InvalidNumber,
}

impl ParseError {
    pub(crate) fn report(&self) -> Diagnostic<()> {
        use ParseErrorCause::*;

        match &self.cause {
            EndOfInput => Diagnostic::error().with_message("unexpected end of input"),
            UnexpectedToken { .. } => Diagnostic::error().with_message("todo"),
            Expected { .. } => Diagnostic::error(),
            ExpectedOneOf { .. } => Diagnostic::error(),
            _ => Diagnostic::error(),
        }
    }
}
