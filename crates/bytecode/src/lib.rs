pub enum Constant {}

// Each opcode is described with e.g (Address, Number) which means that
// first Address followed by a Number will be popped from the stack.
// VM will panic if the popped value is not of an expected type.
#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    // Literals e.g number, string, bool
    Constant(u8),
    // ! (Bool)
    Not,
    // - (Number)
    Neg,
    // + (Number, Number)
    Add,
    // - (Number, Number)
    Sub,
    // / (Number, Number)
    Div,
    // * (Number, Number)
    Mul,
    // ** (Number, Number)
    Pow,
    // √ (Number)
    Sqr,
    // % (Number, Number)
    Mod,
    // == (Any, Any)
    Eq,
    // != (Any, Any)
    Ne,
    // < (Number, Number)
    Lt,
    // <= (Number, Number)
    Le,
    // > (Number, Number)
    Gt,
    // >= (Number, Number)
    Ge,
    // or (Bool, Bool)
    Or,
    // and (Bool, Bool)
    And,
    // jump if false (Usize, Bool)
    Jif,
    // jump forward (Usize)
    Jf,
    // jump back (Usize)
    Jb,
    // return
    Rtr,
    // pop n values from stack (Usize)
    Pop,
    // Get (Address)
    Get,
    // Assign (Address, Any)
    Asg,
    // Call function or method, (Callable)
    Call,
}

#[cfg(test)]

mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
