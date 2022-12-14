use parser::parse::expr::{Expr, ExprKind};

use crate::{BytecodeFrom, BytecodeGenerator, Opcode};

mod atom;
mod binary;
mod unary;

impl BytecodeFrom<Expr> for BytecodeGenerator {
    fn generate(&mut self, expr: Expr) -> crate::BytecodeGenerationResult {
        match *expr.kind {
            ExprKind::Atom(atomic_value) => {
                self.generate(atomic_value)?;
            }
            ExprKind::Binary { lhs, op, rhs } => {
                self.generate(lhs)?;
                self.generate(rhs)?;
                let operator_code = op.kind.into();
                self.write_opcode(operator_code);
            }
            ExprKind::Unary { op, rhs } => {
                self.generate(rhs)?;
                let operator_code = op.kind.into();
                self.write_opcode(operator_code);
            }
            ExprKind::If {
                condition,
                body,
                else_expr,
            } => {
                self.generate(condition)?;
                let jif_patch = self.emit_patch(Opcode::Jif(0));
                self.generate(body)?;
                let jp_patch = self.emit_patch(Opcode::Jp(0));
                self.patch(&jp_patch);
                if let Some(else_expr) = else_expr {
                    self.generate(else_expr)?;
                }
                self.patch(&jif_patch);
            }
            ExprKind::Block { stmts, return_expr } => {}
            ExprKind::While { condition, body } => {}
            ExprKind::Break { return_expr } => {}
            ExprKind::Continue => {}
            ExprKind::Call { callee, args } => {}
            ExprKind::Return { value } => {}
            ExprKind::Array { values } => {}
            ExprKind::Index { target, position } => {}
            ExprKind::Property { target, paths } => {}
            ExprKind::Assignment { target, value } => {}
            ExprKind::Closure { params, body } => {}
            ExprKind::Super => {}
            ExprKind::This => {}
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{BytecodeGenerator, Opcode};

    #[test]
    fn it_patches_opcodes() {
        let mut generator = BytecodeGenerator::new();
        let patch = generator.emit_patch(Opcode::Jif(0));
        assert_eq!(patch.index, 0);
        // Adding some random opcodes to the chunk
        generator.write_opcode(Opcode::Add);
        generator.write_opcode(Opcode::Get);
        // We added some codes but the patched opcode remain the same
        assert_eq!(
            generator.clone().code().chunk.opcodes[patch.index],
            Opcode::Jif(0)
        );
        generator.patch(&patch);
        // After the patch the opcode internal value should be changed to +2
        // because we added two new opcodes and the jump should jump by 2
        assert_eq!(
            generator.clone().code().chunk.opcodes[patch.index],
            Opcode::Jif(2)
        );
    }
}
