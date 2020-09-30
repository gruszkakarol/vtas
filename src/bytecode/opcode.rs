use derive_more::Display;

use crate::parser::Token;

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Display)]
pub enum Opcode {
    // Values
    True,
    False,
    Null,
    Constant(u8),
    // Negation stuff
    Not,
    Negate,
    // binary operators
    Add,
    Subtract,
    Multiply,
    Divide,
    // Comparison
    BangEqual,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    // Jumps
    JumpIfFalse(u8),
    JumpForward(u8),
    JumpBack(u8),
    // Expressions
    Return,
    // Block holds number of variables declared inside to drop
    Block(u8),
    // Side effects
    Print,
    PopN(u8),
    // Variables
    Var(u8),
    VarRef(u8),
    Assign,
}

impl From<Token> for Opcode {
    fn from(token: Token) -> Self {
        match token {
            Token::Plus => Opcode::Add,
            Token::Minus => Opcode::Subtract,
            Token::Star => Opcode::Multiply,
            Token::Divide => Opcode::Divide,
            Token::BangEqual => Opcode::BangEqual,
            Token::Equal => Opcode::Equal,
            Token::Less => Opcode::Less,
            Token::LessEqual => Opcode::LessEqual,
            Token::Greater => Opcode::Greater,
            Token::GreaterEqual => Opcode::GreaterEqual,
            Token::Assign => Opcode::Assign,
            _ => panic!("Can't transform {} into opcode.", token),
        }
    }
}

impl From<bool> for Opcode {
    fn from(bool: bool) -> Self {
        match bool {
            true => Opcode::True,
            false => Opcode::False,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // It's possible to instantiate few opcodes from Token
    #[test]
    fn opcode_from_token() {
        assert_eq!(Opcode::from(Token::Plus), Opcode::Add);
        assert_eq!(Opcode::from(Token::Minus), Opcode::Subtract);
        assert_eq!(Opcode::from(Token::Star), Opcode::Multiply);
        assert_eq!(Opcode::from(Token::Divide), Opcode::Divide);
        assert_eq!(Opcode::from(Token::BangEqual), Opcode::BangEqual);
        assert_eq!(Opcode::from(Token::Equal), Opcode::Equal);
        assert_eq!(Opcode::from(Token::Less), Opcode::Less);
        assert_eq!(Opcode::from(Token::LessEqual), Opcode::LessEqual);
        assert_eq!(Opcode::from(Token::Greater), Opcode::Greater);
        assert_eq!(Opcode::from(Token::GreaterEqual), Opcode::GreaterEqual);
        assert_eq!(Opcode::from(Token::Assign), Opcode::Assign);
    }

    // but not all of them, otherwise it panics.
    // This is an error somewhere in the bytecode generation logic,
    // so there is no better way than panic and let me know.
    #[test]
    #[should_panic]
    fn opcode_from_invalid_token() {
        Opcode::from(Token::OpenBrace);
    }
}
