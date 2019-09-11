use crate::structure::*;
use crate::validation::{Context, ValidationError, ValidationResult};

const BASE: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
enum Operand {
    Value(ValueType),
    Unknown,
}

#[derive(Debug)]
struct ControlFrame {
    label_types: Vec<ValueType>,
    end_types: Vec<ValueType>,
    height: usize,
    unreachable: bool,
}

#[derive(Debug)]
struct ExpressionContext {
    operands: Vec<Operand>,
    frames: Vec<ControlFrame>,
}

impl ExpressionContext {
    fn new() -> ExpressionContext {
        ExpressionContext {
            operands: vec![],
            frames: vec![],
        }
    }

    fn top_frame(&self) -> Result<&ControlFrame, ValidationError> {
        self.frames
            .last()
            .ok_or_else(|| ValidationError::from("Unexpected empty frame stack"))
    }

    fn push_operand(&mut self, operand: Operand) {
        self.operands.push(operand);
    }

    fn pop_operand(&mut self) -> Result<Operand, ValidationError> {
        if self.operands.len() == self.top_frame()?.height && self.top_frame()?.unreachable {
            Ok(Operand::Unknown)
        } else if self.operands.len() == self.top_frame()?.height {
            Err(ValidationError::from("Invalid stack size"))
        } else {
            self.operands
                .pop()
                .ok_or_else(|| ValidationError::from("Unexpected empty operand stack"))
        }
    }

    fn pop_operand_expected(&mut self, expected: &Operand) -> Result<Operand, ValidationError> {
        let actual = self.pop_operand()?;

        if actual == Operand::Unknown {
            Ok(expected.clone())
        } else if *expected == Operand::Unknown {
            Ok(actual)
        } else if actual != *expected {
            Err(ValidationError::from_string(format!(
                "Mismatching type. Expected {:?} but received {:?}",
                expected, actual
            )))
        } else {
            Ok(actual)
        }
    }

    // fn push_operands(&mut self, operands: Vec<Operand>) {
    //     for operand in operands {
    //         self.operands.push(operand);
    //     }
    // }

    fn pop_operands(&mut self, operands: &Vec<Operand>) -> ValidationResult {
        let mut clone = operands.clone();
        clone.reverse();

        for operand in clone {
            self.pop_operand_expected(&operand)?;
        }

        Ok(())
    }

    // fn push_control(&mut self, label_types: Vec<ValueType>, end_types: Vec<ValueType>) {
    //     self.frames.push(ControlFrame {
    //         label_types,
    //         end_types,
    //         height: self.operands.len(),
    //         unreachable: false,
    //     });
    // }

    // fn pop_control(&mut self) -> Result<Vec<ValueType>, ValidationError> {
    //     let frame = self.top_frame()?;
    //     let operand: Vec<Operand> = frame.end_types.iter().map(|t| Operand::Value(*t)).collect();
    //     self.pop_operands(&operand)?;

    //     if self.operands.len() != self.top_frame()?.height {
    //         return Err(ValidationError::from("Mismatching frame height"))
    //     }

    //     // Should be safe to pop the top frame since it has been checked when accessing the top
    //     // frame.
    //     // self.frames.pop().unwrap();
    //     Ok(frame.end_types.clone())
    // }

    fn unreachable(&mut self) -> ValidationResult {
        // let frame = self.frames.last().unwrap();

        // self.operands.resize(frame.height, Operand::Unknown);
        // frame.unreachable = true;

        Ok(())
    }
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-load
fn validate_load_instruction(
    context: &Context,
    expression_context: &mut ExpressionContext,
    memory_args: &MemoryArg,
    value_type: ValueType,
) -> ValidationResult {
    context.get_memory(0)?;

    let bit_width = match value_type {
        ValueType::I32 | ValueType::F32 => 32,
        ValueType::I64 | ValueType::F64 => 64,
    };

    if BASE.pow(memory_args.align) > bit_width / 8 {
        return Err(ValidationError::from("Invalid memory alignment"));
    }

    expression_context.pop_operand_expected(&Operand::Value(ValueType::I32))?;
    expression_context.push_operand(Operand::Value(value_type));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-loadn
fn validate_load_instruction_n(
    context: &Context,
    expression_context: &mut ExpressionContext,
    memory_args: &MemoryArg,
    value_type: ValueType,
    n: u32,
) -> ValidationResult {
    context.get_memory(0)?;

    if BASE.pow(memory_args.align) > (n / 8).into() {
        return Err(ValidationError::from("Invalid memory alignment"));
    }

    expression_context.pop_operand_expected(&Operand::Value(ValueType::I32))?;
    expression_context.push_operand(Operand::Value(value_type));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#id16
fn validate_store_instruction(
    context: &Context,
    expression_context: &mut ExpressionContext,
    memory_args: &MemoryArg,
    value_type: ValueType,
) -> ValidationResult {
    context.get_memory(0)?;

    let bit_width = match value_type {
        ValueType::I32 | ValueType::F32 => 32,
        ValueType::I64 | ValueType::F64 => 64,
    };

    if BASE.pow(memory_args.align) > bit_width / 8 {
        return Err(ValidationError::from("Invalid memory alignment"));
    }

    expression_context.pop_operand_expected(&Operand::Value(ValueType::I32))?;
    expression_context.pop_operand_expected(&Operand::Value(value_type))?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-storen
fn validate_store_instruction_n(
    context: &Context,
    expression_context: &mut ExpressionContext,
    memory_args: &MemoryArg,
    value_type: ValueType,
    n: u32,
) -> ValidationResult {
    context.get_memory(0)?;

    if BASE.pow(memory_args.align) > (n / 8).into() {
        return Err(ValidationError::from("Invalid memory alignment"));
    }

    expression_context.pop_operand_expected(&Operand::Value(ValueType::I32))?;
    expression_context.pop_operand_expected(&Operand::Value(value_type))?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-const
fn validate_const_instruction(
    expression_context: &mut ExpressionContext,
    value_type: ValueType,
) -> ValidationResult {
    expression_context.push_operand(Operand::Value(value_type));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-unop
fn validate_unary_instruction(
    expression_context: &mut ExpressionContext,
    value_type: ValueType,
) -> ValidationResult {
    expression_context.pop_operand_expected(&Operand::Value(value_type))?;
    expression_context.push_operand(Operand::Value(value_type));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-unop
fn validate_binary_instruction(
    expression_context: &mut ExpressionContext,
    value_type: ValueType,
) -> ValidationResult {
    expression_context.pop_operands(&vec![
        Operand::Value(value_type),
        Operand::Value(value_type),
    ])?;
    expression_context.push_operand(Operand::Value(value_type));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-testop
fn validate_test_instruction(
    expression_context: &mut ExpressionContext,
    value_type: ValueType,
) -> ValidationResult {
    expression_context.pop_operand_expected(&Operand::Value(value_type))?;
    expression_context.push_operand(Operand::Value(ValueType::I32));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-testop
fn validate_comparison_instruction(
    expression_context: &mut ExpressionContext,
    value_type: ValueType,
) -> ValidationResult {
    expression_context.pop_operands(&vec![
        Operand::Value(value_type),
        Operand::Value(value_type),
    ])?;
    expression_context.push_operand(Operand::Value(ValueType::I32));
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-testop
fn validate_conversion_instruction(
    expression_context: &mut ExpressionContext,
    input_value_type: ValueType,
    output_value_type: ValueType,
) -> ValidationResult {
    expression_context.pop_operand_expected(&Operand::Value(input_value_type))?;
    expression_context.push_operand(Operand::Value(output_value_type));
    Ok(())
}

fn validate_instruction(
    context: &Context,
    expression_context: &mut ExpressionContext,
    instruction: &Instruction,
) -> ValidationResult {
    match instruction {
        Instruction::Unreachable => {
            expression_context.unreachable()?;
        }
        Instruction::Nop => {}
        Instruction::Block(_block_type, _instructions) => {}
        Instruction::Loop(_block_type, _instructions) => {}
        Instruction::If(_block_type, _if_instructions, _else_instructions) => {}
        Instruction::Br(_label_index) => {}
        Instruction::BrIf(_label_index) => {}
        Instruction::BrTable(_label_indexes, _default_index) => {}
        Instruction::Return => {}
        Instruction::Call(_function_index) => {}
        Instruction::CallIndirect(_function_index) => {}

        Instruction::Drop => {
            expression_context.pop_operand()?;
        }
        Instruction::Select => {
            let t1 = expression_context.pop_operand()?;
            let t2 = expression_context.pop_operand()?;
            expression_context.pop_operand_expected(&Operand::Value(ValueType::I32))?;

            if t1 != t2 {
                return Err(ValidationError::from("Invalid select instruction argument"));
            }

            expression_context.push_operand(t1);
        }

        Instruction::LocalGet(local_index) => {
            let local = context.get_local(*local_index)?;
            expression_context.push_operand(Operand::Value(*local));
        }
        Instruction::LocalSet(local_index) => {
            let local = context.get_local(*local_index)?;
            expression_context.pop_operand_expected(&Operand::Value(*local))?;
        }
        Instruction::LocalTee(local_index) => {
            let local = context.get_local(*local_index)?;
            expression_context.pop_operand_expected(&Operand::Value(*local))?;
            expression_context.push_operand(Operand::Value(*local));
        }
        Instruction::GlobalGet(global_index) => {
            let global = context.get_global(*global_index)?;
            let value_type = global.global_type.value_type;
            expression_context.push_operand(Operand::Value(value_type.clone()));
        }
        Instruction::GlobalSet(global_index) => {
            let global = context.get_global(*global_index)?;

            if global.global_type.mutability != GlobalTypeMutability::Var {
                return Err(ValidationError::from(
                    "Invalid global.set on a non variable global",
                ));
            }

            let value_type = global.global_type.value_type;
            expression_context.pop_operand_expected(&Operand::Value(value_type.clone()))?;
        }

        Instruction::I32Load(memory_args) => {
            validate_load_instruction(context, expression_context, memory_args, ValueType::I32)?;
        }
        Instruction::I64Load(memory_args) => {
            validate_load_instruction(context, expression_context, memory_args, ValueType::I64)?;
        }
        Instruction::F32Load(memory_args) => {
            validate_load_instruction(context, expression_context, memory_args, ValueType::F32)?;
        }
        Instruction::F64Load(memory_args) => {
            validate_load_instruction(context, expression_context, memory_args, ValueType::F64)?;
        }
        Instruction::I32Load8S(memory_args) | Instruction::I32Load8U(memory_args) => {
            validate_load_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I32,
                8,
            )?;
        }
        Instruction::I32Load16S(memory_args) | Instruction::I32Load16U(memory_args) => {
            validate_load_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I32,
                16,
            )?;
        }
        Instruction::I64Load8S(memory_args) | Instruction::I64Load8U(memory_args) => {
            validate_load_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I64,
                8,
            )?;
        }
        Instruction::I64Load16S(memory_args) | Instruction::I64Load16U(memory_args) => {
            validate_load_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I64,
                16,
            )?;
        }
        Instruction::I64Load32S(memory_args) | Instruction::I64Load32U(memory_args) => {
            validate_load_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I64,
                32,
            )?;
        }
        Instruction::I32Store(memory_args) => {
            validate_store_instruction(context, expression_context, memory_args, ValueType::I32)?;
        }
        Instruction::I64Store(memory_args) => {
            validate_store_instruction(context, expression_context, memory_args, ValueType::I64)?;
        }
        Instruction::F32Store(memory_args) => {
            validate_store_instruction(context, expression_context, memory_args, ValueType::F32)?;
        }
        Instruction::F64Store(memory_args) => {
            validate_store_instruction(context, expression_context, memory_args, ValueType::F64)?;
        }
        Instruction::I32Store8(memory_args) => {
            validate_store_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I32,
                8,
            )?;
        }
        Instruction::I32Store16(memory_args) => {
            validate_store_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I32,
                16,
            )?;
        }
        Instruction::I64Store8(memory_args) => {
            validate_store_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I64,
                8,
            )?;
        }
        Instruction::I64Store16(memory_args) => {
            validate_store_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I64,
                16,
            )?;
        }
        Instruction::I64Store32(memory_args) => {
            validate_store_instruction_n(
                context,
                expression_context,
                memory_args,
                ValueType::I64,
                32,
            )?;
        }
        Instruction::MemorySize => {
            context.get_memory(0)?;
            expression_context.push_operand(Operand::Value(ValueType::I32));
        }
        Instruction::MemoryGrow => {
            context.get_memory(0)?;
            expression_context.pop_operand_expected(&Operand::Value(ValueType::I32))?;
            expression_context.push_operand(Operand::Value(ValueType::I32));
        }

        Instruction::I32Const(_value) => {
            validate_const_instruction(expression_context, ValueType::I32)?;
        }
        Instruction::I64Const(_value) => {
            validate_const_instruction(expression_context, ValueType::I64)?;
        }
        Instruction::F32Const(_value) => {
            validate_const_instruction(expression_context, ValueType::F32)?;
        }
        Instruction::F64Const(_value) => {
            validate_const_instruction(expression_context, ValueType::F64)?;
        }

        Instruction::I32Eqz => {
            validate_test_instruction(expression_context, ValueType::I32)?;
        }
        Instruction::I32Eq
        | Instruction::I32Ne
        | Instruction::I32LtS
        | Instruction::I32LtU
        | Instruction::I32GtS
        | Instruction::I32GtU
        | Instruction::I32LeS
        | Instruction::I32LeU
        | Instruction::I32GeS
        | Instruction::I32GeU => {
            validate_comparison_instruction(expression_context, ValueType::I32)?;
        }
        Instruction::I64Eqz => {
            validate_test_instruction(expression_context, ValueType::I64)?;
        }
        Instruction::I64Eq
        | Instruction::I64Ne
        | Instruction::I64LtS
        | Instruction::I64LtU
        | Instruction::I64GtS
        | Instruction::I64GtU
        | Instruction::I64LeS
        | Instruction::I64LeU
        | Instruction::I64GeS
        | Instruction::I64GeU => {
            validate_comparison_instruction(expression_context, ValueType::I64)?;
        }
        Instruction::F32Eq
        | Instruction::F32Ne
        | Instruction::F32Lt
        | Instruction::F32Gt
        | Instruction::F32Le
        | Instruction::F32Ge => {
            validate_comparison_instruction(expression_context, ValueType::F32)?;
        }
        Instruction::F64Eq
        | Instruction::F64Ne
        | Instruction::F64Lt
        | Instruction::F64Gt
        | Instruction::F64Le
        | Instruction::F64Ge => {
            validate_comparison_instruction(expression_context, ValueType::F64)?;
        }

        Instruction::I32Clz | Instruction::I32Ctz | Instruction::I32Popcnt => {
            validate_unary_instruction(expression_context, ValueType::I32)?;
        }
        Instruction::I32Add
        | Instruction::I32Sub
        | Instruction::I32Mul
        | Instruction::I32DivS
        | Instruction::I32DivU
        | Instruction::I32RemS
        | Instruction::I32RemU
        | Instruction::I32And
        | Instruction::I32Or
        | Instruction::I32Xor
        | Instruction::I32Shl
        | Instruction::I32ShrS
        | Instruction::I32ShrU
        | Instruction::I32Rotl
        | Instruction::I32Rotr => {
            validate_binary_instruction(expression_context, ValueType::I32)?;
        }
        Instruction::I64Clz | Instruction::I64Ctz | Instruction::I64Popcnt => {
            validate_unary_instruction(expression_context, ValueType::I64)?;
        }
        Instruction::I64Add
        | Instruction::I64Sub
        | Instruction::I64Mul
        | Instruction::I64DivS
        | Instruction::I64DivU
        | Instruction::I64RemS
        | Instruction::I64RemU
        | Instruction::I64And
        | Instruction::I64Or
        | Instruction::I64Xor
        | Instruction::I64Shl
        | Instruction::I64ShrS
        | Instruction::I64ShrU
        | Instruction::I64Rotl
        | Instruction::I64Rotr => {
            validate_binary_instruction(expression_context, ValueType::I64)?;
        }
        Instruction::F32Abs
        | Instruction::F32Neg
        | Instruction::F32Ceil
        | Instruction::F32Floor
        | Instruction::F32Trunc
        | Instruction::F32Nearest
        | Instruction::F32Sqrt => {
            validate_unary_instruction(expression_context, ValueType::F32)?;
        }
        Instruction::F32Add
        | Instruction::F32Sub
        | Instruction::F32Mul
        | Instruction::F32Div
        | Instruction::F32Min
        | Instruction::F32Max
        | Instruction::F32CopySign => {
            validate_binary_instruction(expression_context, ValueType::F32)?;
        }
        Instruction::F64Abs
        | Instruction::F64Neg
        | Instruction::F64Ceil
        | Instruction::F64Floor
        | Instruction::F64Trunc
        | Instruction::F64Nearest
        | Instruction::F64Sqrt => {
            validate_unary_instruction(expression_context, ValueType::F64)?;
        }
        Instruction::F64Add
        | Instruction::F64Sub
        | Instruction::F64Mul
        | Instruction::F64Div
        | Instruction::F64Min
        | Instruction::F64Max
        | Instruction::F64CopySign => {
            validate_unary_instruction(expression_context, ValueType::F64)?;
        }

        Instruction::I32WrapI64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::I32)?;
        }
        Instruction::I32TruncSF32 => {
            validate_conversion_instruction(expression_context, ValueType::F32, ValueType::I32)?;
        }
        Instruction::I32TruncUF32 => {
            validate_conversion_instruction(expression_context, ValueType::F32, ValueType::I32)?;
        }
        Instruction::I32TruncSF64 => {
            validate_conversion_instruction(expression_context, ValueType::F64, ValueType::I32)?;
        }
        Instruction::I32TruncUF64 => {
            validate_conversion_instruction(expression_context, ValueType::F64, ValueType::I32)?;
        }
        Instruction::I64ExtendSI32 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::I32)?;
        }
        Instruction::I64ExtendUI32 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::I32)?;
        }
        Instruction::I64TruncSF32 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F32)?;
        }
        Instruction::I64TruncUF32 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F32)?;
        }
        Instruction::I64TruncSF64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F64)?;
        }
        Instruction::I64TruncUF64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F64)?;
        }
        Instruction::F32ConvertSI32 => {
            validate_conversion_instruction(expression_context, ValueType::I32, ValueType::F32)?;
        }
        Instruction::F32ConvertUI32 => {
            validate_conversion_instruction(expression_context, ValueType::I32, ValueType::F32)?;
        }
        Instruction::F32ConvertSI64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F32)?;
        }
        Instruction::F32ConvertUI64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F32)?;
        }
        Instruction::F32DemoteF64 => {
            validate_conversion_instruction(expression_context, ValueType::F64, ValueType::F32)?;
        }
        Instruction::F64ConvertSI32 => {
            validate_conversion_instruction(expression_context, ValueType::I32, ValueType::F64)?;
        }
        Instruction::F64ConvertUI32 => {
            validate_conversion_instruction(expression_context, ValueType::I32, ValueType::F64)?;
        }
        Instruction::F64ConvertSI64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F64)?;
        }
        Instruction::F64ConvertUI64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F64)?;
        }
        Instruction::F64PromoteF32 => {
            validate_conversion_instruction(expression_context, ValueType::F32, ValueType::F64)?;
        }

        Instruction::I32ReinterpretF32 => {
            validate_conversion_instruction(expression_context, ValueType::F32, ValueType::I32)?;
        }
        Instruction::I64ReinterpretF64 => {
            validate_conversion_instruction(expression_context, ValueType::F64, ValueType::I64)?;
        }
        Instruction::F32ReinterpretI32 => {
            validate_conversion_instruction(expression_context, ValueType::I32, ValueType::F32)?;
        }
        Instruction::F64ReinterpretI64 => {
            validate_conversion_instruction(expression_context, ValueType::I64, ValueType::F64)?;
        }
    };

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#id31
pub fn validate_expression(
    context: &Context,
    expression: &Expression,
    _return_types: Vec<ValueType>,
) -> ValidationResult {
    let mut expression_context = ExpressionContext::new();

    for instruction in expression {
        validate_instruction(&context, &mut expression_context, &instruction)?;
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#constant-expressions
pub fn validate_constant_expression(
    context: &Context,
    expression: &Expression,
) -> ValidationResult {
    for instruction in expression {
        match instruction {
            Instruction::I32Const(_) => {}
            Instruction::I64Const(_) => {}
            Instruction::F32Const(_) => {}
            Instruction::F64Const(_) => {}
            Instruction::GlobalGet(global) => {
                context.get_global(*global)?;
            }
            _ => {
                return Err(ValidationError::from(
                    "Invalid instruction in constant expression",
                ))
            }
        }
    }

    Ok(())
}
