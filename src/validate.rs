use super::types::*;

const BASE: u64 = 2;

#[derive(Debug)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    fn from(message: &str) -> ValidationError {
        ValidationError {
            message: String::from(message),
        }
    }

    fn from_string(message: String) -> ValidationError {
        ValidationError { message }
    }
}

pub type ValidationResult = Result<(), ValidationError>;

#[derive(Debug)]
struct Context<'a> {
    function_types: &'a Vec<FunctionType>,
    functions: &'a Vec<Function>,
    tables: &'a Vec<Table>,
    memories: &'a Vec<Memory>,
    globals: &'a Vec<Global>,
    elements: &'a Vec<Element>,
    locals: Vec<ValueType>,
}

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
            .ok_or(ValidationError::from("Unexpected empty frame stack"))
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
                .ok_or(ValidationError::from("Unexpected empty operand stack"))
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

    fn push_operands(&mut self, operands: Vec<Operand>) {
        for operand in operands {
            self.operands.push(operand);
        }
    }

    fn pop_operands(&mut self, operands: &Vec<Operand>) -> ValidationResult {
        let mut clone = operands.clone();
        clone.reverse();

        for operand in clone {
            self.pop_operand_expected(&operand)?;
        }

        Ok(())
    }

    fn push_control(&mut self, label_types: Vec<ValueType>, end_types: Vec<ValueType>) {
        self.frames.push(ControlFrame {
            label_types,
            end_types,
            height: self.operands.len(),
            unreachable: false,
        });
    }

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

fn get_function_type<'a>(
    context: &'a Context,
    function_type_index: u32,
) -> Result<&'a FunctionType, ValidationError> {
    context
        .function_types
        .get(function_type_index as usize)
        .ok_or(ValidationError::from("Invalid function type reference"))
}

fn get_function<'a>(
    context: &'a Context,
    function_index: u32,
) -> Result<&'a Function, ValidationError> {
    context
        .functions
        .get(function_index as usize)
        .ok_or(ValidationError::from("Invalid function reference"))
}

fn get_table<'a>(context: &'a Context, table_index: u32) -> Result<&'a Table, ValidationError> {
    context
        .tables
        .get(table_index as usize)
        .ok_or(ValidationError::from("Invalid table reference"))
}

fn get_memory<'a>(context: &'a Context, memory_index: u32) -> Result<&'a Memory, ValidationError> {
    context
        .memories
        .get(memory_index as usize)
        .ok_or(ValidationError::from("Invalid memory reference"))
}

fn get_global<'a>(context: &'a Context, global_index: u32) -> Result<&'a Global, ValidationError> {
    context
        .globals
        .get(global_index as usize)
        .ok_or(ValidationError::from("Invalid global reference"))
}

fn get_local<'a>(context: &'a Context, local_index: u32) -> Result<&'a ValueType, ValidationError> {
    context
        .locals
        .get(local_index as usize)
        .ok_or(ValidationError::from("Invalid local reference"))
}

// https://webassembly.github.io/spec/core/valid/types.html#limits
// TODO: Should we validate the range? Are we doing non-sense type casting is the range is always
// 2^32
fn validate_limits(limits: &Limits, range: u64) -> ValidationResult {
    if limits.min as u64 > range {
        return Err(ValidationError::from("Limit minimum is above valid range"));
    }

    match limits.max {
        Some(max) => {
            if (max as u64) > range {
                return Err(ValidationError::from("Limit maximum is above valid range"));
            }
            if max < limits.min {
                return Err(ValidationError::from("Limit maximum is below minimum"));
            }

            Ok(())
        }
        None => Ok(()),
    }?;

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-functype
fn validate_function_type(function_type: &FunctionType) -> ValidationResult {
    let (_params, returns) = function_type;

    if returns.len() > 1 {
        return Err(ValidationError::from("Too many return value"));
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-tabletype
fn validate_table_type(table_type: &TableType) -> ValidationResult {
    // TODO: Find a better way to express this
    validate_limits(&table_type.limits, BASE.pow(32))?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-memtype
fn validate_memory_type(memory_type: &MemoryType) -> ValidationResult {
    validate_limits(&memory_type.limits, BASE.pow(16))?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-globaltype
fn validate_global_type(_global_type: &GlobalType) -> ValidationResult {
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#valid-load
fn validate_load_instruction(
    context: &Context,
    expression_context: &mut ExpressionContext,
    memory_args: &MemoryArg,
    value_type: ValueType,
) -> ValidationResult {
    get_memory(context, 0)?;

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
    get_memory(context, 0)?;

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
    get_memory(context, 0)?;

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
    n: u32
) -> ValidationResult {
    get_memory(context, 0)?;

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
        Instruction::Block(block_type, instructions) => {}
        Instruction::Loop(block_type, instructions) => {}
        Instruction::If(block_type, if_instructions, else_instructions) => {}
        Instruction::Br(label_index) => {}
        Instruction::BrIf(label_index) => {}
        Instruction::BrTable(label_indexes, default_index) => {}
        Instruction::Return => {}
        Instruction::Call(function_index) => {}
        Instruction::CallIndirect(function_index) => {}

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
            let local = get_local(context, *local_index)?;
            expression_context.push_operand(Operand::Value(local.clone()));
        }
        Instruction::LocalSet(local_index) => {
            let local = get_local(context, *local_index)?;
            expression_context.pop_operand_expected(&Operand::Value(local.clone()))?;
        }
        Instruction::LocalTee(local_index) => {
            let local = get_local(context, *local_index)?;
            expression_context.pop_operand_expected(&Operand::Value(local.clone()))?;
            expression_context.push_operand(Operand::Value(local.clone()));
        }
        Instruction::GlobalGet(global_index) => {
            let global = get_global(context, *global_index)?;
            let value_type = global.global_type.value_type;
            expression_context.push_operand(Operand::Value(value_type.clone()));
        }
        Instruction::GlobalSet(global_index) => {
            let global = get_global(context, *global_index)?;

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
            validate_store_instruction_n(context, expression_context, memory_args, ValueType::I32, 8)?;
        }
        Instruction::I32Store16(memory_args) => {
            validate_store_instruction_n(context, expression_context, memory_args, ValueType::I32, 16)?;
        }
        Instruction::I64Store8(memory_args) => {
            validate_store_instruction_n(context, expression_context, memory_args, ValueType::I64, 8)?;
        }
        Instruction::I64Store16(memory_args) => {
            validate_store_instruction_n(context, expression_context, memory_args, ValueType::I64, 16)?;
        }
        Instruction::I64Store32(memory_args) => {
            validate_store_instruction_n(context, expression_context, memory_args, ValueType::I64, 32)?;
        }
        Instruction::MemorySize => {
            get_memory(context, 0)?;
            expression_context.push_operand(Operand::Value(ValueType::I32));
        }
        Instruction::MemoryGrow => {
            get_memory(context, 0)?;
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
fn validate_expression(
    context: &Context,
    expression: &Expression,
    return_types: Vec<ValueType>,
) -> ValidationResult {
    let mut expression_context = ExpressionContext::new();

    for instruction in expression {
        validate_instruction(&context, &mut expression_context, &instruction)?;
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/instructions.html#constant-expressions
fn validate_constant_expression(context: &Context, expression: &Expression) -> ValidationResult {
    for instruction in expression {
        match instruction {
            Instruction::I32Const(_) => {}
            Instruction::I64Const(_) => {}
            Instruction::F32Const(_) => {}
            Instruction::F64Const(_) => {}
            Instruction::GlobalGet(global) => {
                get_global(context, *global)?;
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

// https://webassembly.github.io/spec/core/valid/modules.html#valid-func
fn validate_function(context: &Context, function: &Function) -> ValidationResult {
    let function_type = get_function_type(context, function.function_type)?;

    let (_params, _returns) = function_type;
    // TODO: Validate expression

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#tables
fn validate_table(table: &Table) -> ValidationResult {
    validate_table_type(&table.table_type)?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-mem
fn validate_memory(memory: &Memory) -> ValidationResult {
    validate_memory_type(&memory.memory_type)?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-global
fn validate_global(context: &Context, global: &Global) -> ValidationResult {
    validate_global_type(&global.global_type)?;

    validate_expression(context, &global.init, vec![global.global_type.value_type])?;
    validate_constant_expression(context, &global.init)?;

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-elem
fn validate_element(context: &Context, element: &Element) -> ValidationResult {
    get_table(context, element.table)?;

    // No need to validate the table type, since there is only one type of element types in table
    // right now.

    validate_expression(context, &element.offset, vec![ValueType::I32])?;
    validate_constant_expression(context, &element.offset)?;

    for init in &element.init {
        get_function(context, *init)?;
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-data
fn validate_data(context: &Context, data: &Data) -> ValidationResult {
    get_memory(context, data.data)?;

    validate_expression(context, &data.offset, vec![ValueType::I32])?;
    validate_constant_expression(context, &data.offset)?;

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-start
fn validate_start(context: &Context, start: &StartFunction) -> ValidationResult {
    let function = get_function(context, start.function)?;

    // Function type have been validated previously, we should not worry to unwrap the value.
    let (params, returns) = get_function_type(context, function.function_type)?;
    if !params.is_empty() || !returns.is_empty() {
        return Err(ValidationError::from("Invalid start function"));
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-import
fn validate_import(context: &Context, import: &Import) -> ValidationResult {
    match &import.descriptor {
        ImportDescriptor::Function(function) => {
            get_function(context, *function)?;
            Ok(())
        }
        ImportDescriptor::Table(table_type) => validate_table_type(table_type),
        ImportDescriptor::Memory(memory_type) => validate_memory_type(memory_type),
        ImportDescriptor::Global(global_type) => validate_global_type(global_type),
    }
}

// https://webassembly.github.io/spec/core/valid/modules.html#exports
fn validate_export(context: &Context, export: &Export) -> ValidationResult {
    match &export.descriptor {
        ExportDescriptor::Function(function) => {
            get_function(context, *function)?;
        }
        ExportDescriptor::Table(table) => {
            get_table(context, *table)?;
        }
        ExportDescriptor::Memory(memory) => {
            get_memory(context, *memory)?;
        }
        ExportDescriptor::Global(global) => {
            get_global(context, *global)?;
        }
    };

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-module
pub fn validate(module: &Module) -> ValidationResult {
    let context = Context {
        function_types: &module.function_types,
        functions: &module.functions,
        tables: &module.tables,
        memories: &module.memories,
        globals: &module.globals,
        elements: &module.elements,
        locals: vec![],
    };

    for function_type in &module.function_types {
        validate_function_type(&function_type)?;
    }
    for function in &module.functions {
        validate_function(&context, &function)?;
    }
    for table in &module.tables {
        validate_table(&table)?;
    }
    for memory in &module.memories {
        validate_memory(&memory)?;
    }
    for global in &module.globals {
        // TODO: Use custom context
        validate_global(&context, &global)?;
    }
    for element in &module.elements {
        validate_element(&context, &element)?;
    }
    for data in &module.data {
        validate_data(&context, &data)?;
    }
    match &module.start {
        Some(start) => Ok(validate_start(&context, &start)?),
        None => Ok(()),
    }?;
    for import in &module.imports {
        validate_import(&context, &import)?;
    }
    for export in &module.exports {
        validate_export(&context, &export)?;
    }

    if module.tables.len() > 1 {
        return Err(ValidationError::from("Too many tables"));
    }
    if module.memories.len() > 1 {
        return Err(ValidationError::from("Too many memories"));
    }

    for i in 0..module.exports.len() {
        for j in (i + 1)..module.exports.len() {
            if module.exports[i].name == module.exports[j].name {
                return Err(ValidationError::from("Duplicate export names"));
            }
        }
    }

    Ok(())
}
