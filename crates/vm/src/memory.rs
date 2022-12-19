use bytecode::MemoryAddress;
use common::ProgramText;

use crate::{
    gravitas_std::STD_FUNCTIONS, runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue,
    OperationResult, VM,
};

impl VM {
    pub(crate) fn op_pop(&mut self, amount: usize) -> OperationResult {
        for _ in 0..amount as usize {
            self.pop_operand()?;
        }

        Ok(())
    }

    pub(crate) fn assign_value(
        &mut self,
        value: RuntimeValue,
        address: MemoryAddress,
    ) -> OperationResult {
        let stack_start = self.current_frame().stack_start;

        match address {
            MemoryAddress::Local(local_address) => {
                self.operands[stack_start + local_address as usize] = value;
            }
            _ => unimplemented!(),
        }
        Ok(())
    }

    pub(crate) fn op_asg(&mut self) -> OperationResult {
        let to_assign = self.pop_operand()?;
        let address = self.pop_address()?;
        self.assign_value(to_assign, address)?;

        Ok(())
    }

    pub(crate) fn get_local_variable(&mut self, local_address: usize) -> OperationResult {
        let stack_start = self.current_frame().stack_start;

        match self
            .operands
            .get(stack_start + local_address as usize)
            .cloned()
        {
            Some(value) => {
                self.operands.push(value);
                Ok(())
            }
            None => self.error(RuntimeErrorCause::StackOverflow),
        }
    }

    pub(crate) fn get_global_variable(&mut self, name: ProgramText) -> OperationResult {
        let built_in = STD_FUNCTIONS
            .get(&*name)
            .expect("This global variable is not defined.");
        self.operands.push(built_in.clone().into());
        Ok(())
    }

    pub(crate) fn op_get(&mut self) -> OperationResult {
        let address = self.pop_address()?;
        // TODO: move to util function
        match address {
            MemoryAddress::Local(stack_address) => self.get_local_variable(stack_address),
            MemoryAddress::Global(name) => self.get_global_variable(name),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{runtime_value::RuntimeValue, test::new_vm, OperationResult};
    use bytecode::{
        chunk::{Chunk, Constant},
        MemoryAddress, Opcode,
    };

    #[test]
    fn op_pop() -> OperationResult {
        let mut vm = new_vm(Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Pop(3),
            ],
            vec![
                Constant::Bool(true),
                Constant::Bool(true),
                Constant::Bool(true),
            ],
        ));

        // let's push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        vm.tick()?;

        assert_eq!(vm.operands.len(), 3);

        vm.tick()?;

        // Pop(3) will pop 3 operands from the stack
        // so after the operation finishes the operands stack length will be equal to 0
        assert_eq!(vm.operands.len(), 0);

        Ok(())
    }

    #[test]
    fn op_get() -> OperationResult {
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Get],
            vec![
                Constant::Bool(true),
                Constant::MemoryAddress(MemoryAddress::Local(0)),
            ],
        ));

        // push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        // execute get
        vm.tick()?;
        // only Constant::Bool(true) should be present on the stack after it got pushed back there
        let leftover_value = vm.operands[0].clone();

        assert!(leftover_value
            .eq(&RuntimeValue::Bool(true), &mut vm)
            .unwrap());

        Ok(())
    }

    #[test]
    fn op_asg() -> OperationResult {
        let mut vm = new_vm(Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Asg,
            ],
            vec![
                Constant::Number(127.0),
                Constant::MemoryAddress(MemoryAddress::Local(0)),
                Constant::Number(7.0),
            ],
        ));

        // push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        vm.tick()?;

        // the first operand on the stack is the initial value of the variable
        let first_operand = vm.operands[0].clone();
        assert!(first_operand
            .eq(&RuntimeValue::Number(127.0), &mut vm)
            .unwrap());

        // execute Opcode::Asg
        vm.tick()?;

        // but after the execution the first operand on the stack will change to the assigned value
        let assigned_value = vm.operands[0].clone();
        assert!(assigned_value
            .eq(&RuntimeValue::Number(7.0), &mut vm)
            .unwrap());

        Ok(())
    }
}
