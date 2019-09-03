// TODO: Remove once all the properties are exposed.
// Right now there is a lot of warnings because the struct properties are never used.
#![allow(dead_code)]

use std::env;
use std::fs;
use std::io;

// TODO: Understand how this io package work!
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(p) => p,
        _ => panic!("No file argument found"),
    };

    // Error handling
    let file = fs::read(filename)?;
    decode(file);

    Ok(())
}

const SECTION_ID_CUSTOM: u8 = 0;
const SECTION_ID_TYPE: u8 = 1;
const SECTION_ID_IMPORT: u8 = 2;
const SECTION_ID_FUNCTION: u8 = 3;
const SECTION_ID_TABLE: u8 = 4;
const SECTION_ID_MEMORY: u8 = 5;
const SECTION_ID_GLOBAL: u8 = 6;
const SECTION_ID_EXPORT: u8 = 7;
const SECTION_ID_START: u8 = 8;
const SECTION_ID_ELEMENT: u8 = 9;
const SECTION_ID_CODE: u8 = 10;
const SECTION_ID_DATA: u8 = 11;

// TODO: Understand why Copy and Clone are always applied at the same time.
// More details: https://doc.rust-lang.org/std/marker/trait.Copy.html
#[derive(Debug, Copy, Clone)]
enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

type FunctionType = (Vec<ValueType>, Vec<ValueType>);

#[derive(Debug)]
struct Limits {
    min: u32,
    max: Option<u32>,
}

#[derive(Debug)]
enum ElementType {
    FuncRef,
}

#[derive(Debug)]
struct MemoryType {
    limits: Limits,
}

#[derive(Debug)]
struct TableType {
    limits: Limits,
    element_type: ElementType,
}

#[derive(Debug)]
enum GlobalTypeMutability {
    Const,
    Var,
}

#[derive(Debug)]
struct GlobalType {
    value_type: ValueType,
    mutability: GlobalTypeMutability,
}

#[derive(Debug, Copy, Clone)]
enum Index {
    Type(u32),
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
    Local(u32),
    Label(u32),
}

#[derive(Debug)]
struct Table {
    table_type: TableType,
}

#[derive(Debug)]
struct Memory {
    memory_type: MemoryType,
}

#[derive(Debug)]
struct Global {
    global_type: GlobalType,
    init: Expression,
}

#[derive(Debug)]
struct Function {
    function_type: Index,
    locals: Vec<ValueType>,
    body: Expression,
}

#[derive(Debug)]
struct StartFunction {
    function: Index,
}

#[derive(Debug)]
struct Element {
    table: Index,
    offset: Expression,
    init: Vec<Index>,
}

#[derive(Debug)]
struct Data {
    data: Index,
    offset: Expression,
    init: Vec<u8>,
}

#[derive(Debug)]
struct Export {
    name: String,
    descriptor: Index,
}

#[derive(Debug)]
struct Import {
    module: String,
    name: String,
    descriptor: Index,
}

#[derive(Debug, Copy, Clone)]
enum BlockType {
    Void,
    Return(ValueType),
}

#[derive(Debug, Copy, Clone)]
struct MemoryArg {
    align: u32,
    offset: u32,
}

#[derive(Debug, Clone)]
enum Instruction {
    // Control flow instructions
    Unreachable,
    Nop,
    Block(BlockType, Vec<Instruction>),
    Loop(BlockType, Vec<Instruction>),
    If(BlockType, Vec<Instruction>, Option<Vec<Instruction>>),
    Br(Index),
    BrIf(Index),
    BrTable(Vec<Index>, Index),
    Return,
    Call(Index),
    CallIndirect(Index),

    // Parametric instructions
    Drop,
    Select,

    // Variable instructions
    LocalGet(Index),
    LocalSet(Index),
    LocalTee(Index),
    GlobalGet(Index),
    GlobalSet(Index),

    // Memory instructions
    I32Load(MemoryArg),
    I64Load(MemoryArg),
    F32Load(MemoryArg),
    F64Load(MemoryArg),
    I32Load8S(MemoryArg),
    I32Load8U(MemoryArg),
    I32Load16S(MemoryArg),
    I32Load16U(MemoryArg),
    I64Load8S(MemoryArg),
    I64Load8U(MemoryArg),
    I64Load16S(MemoryArg),
    I64Load16U(MemoryArg),
    I64Load32S(MemoryArg),
    I64Load32U(MemoryArg),
    I32Store(MemoryArg),
    I64Store(MemoryArg),
    F32Store(MemoryArg),
    F64Store(MemoryArg),
    I32Store8(MemoryArg),
    I32Store16(MemoryArg),
    I64Store8(MemoryArg),
    I64Store16(MemoryArg),
    I64Store32(MemoryArg),
    MemorySize,
    MemoryGrow,

    // Constants instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    // Comparison operators
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    // Numeric operators
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32CopySign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64CopySign,

    // Conversions
    I32WrapI64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,

    // Reinterpretations
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

type Expression = Vec<Instruction>;

#[derive(Debug)]
struct Module {
    function_types: Vec<FunctionType>,
    functions: Vec<Function>,
    tables: Vec<Table>,
    memories: Vec<Memory>,
    globals: Vec<Global>,
    elements: Vec<Element>,
    data: Vec<Data>,
    start: Option<StartFunction>,
    imports: Vec<Import>,
    exports: Vec<Export>,
}

struct Decoder {
    bytes: Vec<u8>,
    offset: usize,
}

impl Decoder {
    fn eat_byte(&mut self) -> u8 {
        let ret = self.pick_byte().expect("Unexpected end of file");

        self.offset += 1;
        ret
    }

    fn pick_byte(&self) -> Option<u8> {
        if self.offset < self.bytes.len() {
            Some(self.bytes[self.offset])
        } else {
            None
        }
    }
}

fn decode_unsigned_leb_128(decoder: &mut Decoder) -> u64 {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        let byte = decoder.eat_byte() as u64;

        // Extract the low order 7 bits of byte, left shift the byte and add them to the current
        // result.
        result = result | ((byte & 0x7f) << (shift * 7));

        // Increase the shift by one.
        shift += 1;

        // Repeat until the highest order bit (0x80) is 0.
        if (byte & 0x80) != 0x80 {
            break;
        }
    }

    // TODO: validate

    result
}

fn decode_u32(decoder: &mut Decoder) -> u32 {
    decode_unsigned_leb_128(decoder) as u32
}

fn decode_i32(decoder: &mut Decoder) -> i32 {
    // TODO: Fix me
    decode_unsigned_leb_128(decoder) as i32
}

fn decode_i64(decoder: &mut Decoder) -> i64 {
    // TODO: Fix me
    decode_unsigned_leb_128(decoder) as i64
}

fn decode_f32(decoder: &mut Decoder) -> f32 {
    panic!("Not implemented")
}

fn decode_f64(decoder: &mut Decoder) -> f64 {
    panic!("Not implemented")
}

/// https://webassembly.github.io/spec/core/binary/values.html#binary-name
///
/// The higher bits in the first byte contains a mask describing the number of byte encoding the
/// character. In UTF-8 characters can be encoded over 1 to 4 bytes.
fn decode_name(decoder: &mut Decoder) -> String {
    let mut chars = Vec::new();

    let vector_size = decode_u32(decoder);
    for _ in 0..vector_size {
        let byte1 = decoder.eat_byte();

        // 1 byte sequence with no continuation byte
        // [0xxxxxxx]
        if (byte1 & 0x80) == 0 {
            chars.push(byte1);
        }
        // 2 bytes sequence
        // [110xxxxx, 10xxxxxx]
        else if (byte1 & 0xe0) == 0xc0 {
            let byte2 = decoder.eat_byte();
            chars.push(((byte1 & 0x1f) << 6) | byte2);
        }
        // // 3 bytes sequence
        // // [1110xxxx, 10xxxxxx, 10xxxxxx]
        // if (byte1 & 0xf0) == 0xe0 {
        //     let byte2 = decoder.eat_byte();
        //     let byte3 = decoder.eat_byte();
        //     chars.push(((byte1 & 0x0f) << 12) | (byte2 << 6) | byte3);
        // }

        // // 4 bytes sequence
        // // [11110xxx, 10xxxxxx, 10xxxxxx, 10xxxxxx]
        // if (byte1 & 0xf8) == 0xf0 {
        //     let byte2 = decoder.eat_byte();
        //     let byte3 = decoder.eat_byte();
        //     let byte4 = decoder.eat_byte();
        //     chars.push(
        //         ((byte1 & 0x07) << 18) | (byte2 << 12) | (byte3 << 6) | byte4
        //     );
        // }
        else {
            panic!("Invalid utf-8 encoding")
        }
    }

    // TODO: Better error handling
    String::from_utf8(chars).unwrap()
}

/// https://webassembly.github.io/spec/core/binary/types.html#limits
fn decode_limits(decoder: &mut Decoder) -> Limits {
    match decoder.eat_byte() {
        0x00 => Limits {
            min: decode_u32(decoder),
            max: None,
        },
        0x01 => Limits {
            min: decode_u32(decoder),
            max: Some(decode_u32(decoder)),
        },
        _ => panic!("Invalid limit"),
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#value-types
fn decode_value_type(decoder: &mut Decoder) -> ValueType {
    match decoder.eat_byte() {
        0x7F => ValueType::I32,
        0x7E => ValueType::I64,
        0x7D => ValueType::F32,
        0x7C => ValueType::F64,
        _ => panic!("Invalid value type"),
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#binary-functype
fn decode_function_type(decoder: &mut Decoder) -> FunctionType {
    assert!(decoder.eat_byte() == 0x60, "Invalid function type prefix");

    let mut params = Vec::new();
    let mut results = Vec::new();

    let params_vector_size = decode_u32(decoder);
    for _ in 0..params_vector_size {
        let value_type = decode_value_type(decoder);
        params.push(value_type);
    }

    let results_vector_size = decode_u32(decoder);
    for _ in 0..results_vector_size {
        let value_type = decode_value_type(decoder);
        results.push(value_type);
    }

    (params, results)
}

/// https://webassembly.github.io/spec/core/binary/types.html#binary-globaltype
fn decode_global_type(decoder: &mut Decoder) -> GlobalType {
    GlobalType {
        value_type: decode_value_type(decoder),
        mutability: match decoder.eat_byte() {
            0x00 => GlobalTypeMutability::Const,
            0x01 => GlobalTypeMutability::Var,
            _ => panic!("Invalid global type mutability"),
        },
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#binary-blocktype
fn decode_block_type(decoder: &mut Decoder) -> BlockType {
    match decoder.eat_byte() {
        0x40 => BlockType::Void,
        _ => BlockType::Return(decode_value_type(decoder)),
    }
}

/// https://webassembly.github.io/spec/core/binary/instructions.html#binary-memarg
fn decode_memory_arg(decoder: &mut Decoder) -> MemoryArg {
    MemoryArg {
        align: decode_u32(decoder),
        offset: decode_u32(decoder),
    }
}

/// https://webassembly.github.io/spec/core/binary/instructions.html#instructions
fn decode_instruction(decoder: &mut Decoder) -> Instruction {
    match decoder.eat_byte() {
        0x00 => Instruction::Unreachable,
        0x01 => Instruction::Nop,
        0x02 => {
            let block_type = decode_block_type(decoder);
            let mut instructions = Vec::new();
            while decoder.pick_byte().unwrap() != 0x0B {
                instructions.push(decode_instruction(decoder));
            }

            decoder.eat_byte(); // end
            Instruction::Block(block_type, instructions)
        }
        0x03 => {
            let block_type = decode_block_type(decoder);

            let mut instructions = Vec::new();
            while decoder.pick_byte().unwrap() != 0x0B {
                instructions.push(decode_instruction(decoder));
            }

            decoder.eat_byte(); // end
            Instruction::Loop(block_type, instructions)
        }
        0x04 => {
            let block_type = decode_block_type(decoder);

            let mut if_instructions = Vec::new();
            while decoder.pick_byte().unwrap() != 0x0B && decoder.pick_byte().unwrap() != 0x05 {
                if_instructions.push(decode_instruction(decoder));
            }

            let else_instructions = if decoder.pick_byte().unwrap() == 0x05 {
                let mut else_instructions = Vec::new();

                while decoder.pick_byte().unwrap() != 0x0B {
                    else_instructions.push(decode_instruction(decoder));
                }

                Some(else_instructions)
            } else {
                None
            };

            decoder.eat_byte(); // end
            Instruction::If(block_type, if_instructions, else_instructions)
        }
        0x0C => Instruction::Br(Index::Label(decode_u32(decoder))),
        0x0D => Instruction::BrIf(Index::Label(decode_u32(decoder))),
        0x0E => {
            let mut labels = Vec::new();

            let vector_size = decode_u32(decoder);
            for _ in 0..vector_size {
                labels.push(Index::Label(decode_u32(decoder)));
            }

            let default_label = Index::Label(decode_u32(decoder));

            Instruction::BrTable(labels, default_label)
        }
        0x0F => Instruction::Return,
        0x10 => Instruction::Call(Index::Function(decode_u32(decoder))),
        0x11 => Instruction::CallIndirect(Index::Function(decode_u32(decoder))),

        0x1A => Instruction::Drop,
        0x1B => Instruction::Select,

        0x20 => Instruction::LocalGet(Index::Local(decode_u32(decoder))),
        0x21 => Instruction::LocalSet(Index::Local(decode_u32(decoder))),
        0x22 => Instruction::LocalTee(Index::Local(decode_u32(decoder))),
        0x23 => Instruction::GlobalGet(Index::Global(decode_u32(decoder))),
        0x24 => Instruction::GlobalGet(Index::Global(decode_u32(decoder))),

        0x28 => Instruction::I32Load(decode_memory_arg(decoder)),
        0x29 => Instruction::I64Load(decode_memory_arg(decoder)),
        0x2a => Instruction::F32Load(decode_memory_arg(decoder)),
        0x2b => Instruction::F64Load(decode_memory_arg(decoder)),
        0x2c => Instruction::I32Load8S(decode_memory_arg(decoder)),
        0x2d => Instruction::I32Load8U(decode_memory_arg(decoder)),
        0x2e => Instruction::I32Load16S(decode_memory_arg(decoder)),
        0x2f => Instruction::I32Load16U(decode_memory_arg(decoder)),
        0x30 => Instruction::I64Load8S(decode_memory_arg(decoder)),
        0x31 => Instruction::I64Load8U(decode_memory_arg(decoder)),
        0x32 => Instruction::I64Load16S(decode_memory_arg(decoder)),
        0x33 => Instruction::I64Load16U(decode_memory_arg(decoder)),
        0x34 => Instruction::I64Load32S(decode_memory_arg(decoder)),
        0x35 => Instruction::I64Load32U(decode_memory_arg(decoder)),
        0x36 => Instruction::I32Store(decode_memory_arg(decoder)),
        0x37 => Instruction::I64Store(decode_memory_arg(decoder)),
        0x38 => Instruction::F32Store(decode_memory_arg(decoder)),
        0x39 => Instruction::F64Store(decode_memory_arg(decoder)),
        0x3a => Instruction::I32Store8(decode_memory_arg(decoder)),
        0x3b => Instruction::I32Store16(decode_memory_arg(decoder)),
        0x3c => Instruction::I64Store8(decode_memory_arg(decoder)),
        0x3d => Instruction::I64Store16(decode_memory_arg(decoder)),
        0x3e => Instruction::I64Store32(decode_memory_arg(decoder)),

        0x41 => Instruction::I32Const(decode_i32(decoder)),
        0x42 => Instruction::I64Const(decode_i64(decoder)),
        0x43 => Instruction::F32Const(decode_f32(decoder)),
        0x44 => Instruction::F64Const(decode_f64(decoder)),

        0x45 => Instruction::I32Eqz,
        0x46 => Instruction::I32Eq,
        0x47 => Instruction::I32Ne,
        0x48 => Instruction::I32LtS,
        0x49 => Instruction::I32LtU,
        0x4a => Instruction::I32GtS,
        0x4b => Instruction::I32GtU,
        0x4c => Instruction::I32LeS,
        0x4d => Instruction::I32LeU,
        0x4e => Instruction::I32GeS,
        0x4f => Instruction::I32GeU,
        0x50 => Instruction::I64Eqz,
        0x51 => Instruction::I64Eq,
        0x52 => Instruction::I64Ne,
        0x53 => Instruction::I64LtS,
        0x54 => Instruction::I64LtU,
        0x55 => Instruction::I64GtS,
        0x56 => Instruction::I64GtU,
        0x57 => Instruction::I64LeS,
        0x58 => Instruction::I64LeU,
        0x59 => Instruction::I64GeS,
        0x5a => Instruction::I64GeU,
        0x5b => Instruction::F32Eq,
        0x5c => Instruction::F32Ne,
        0x5d => Instruction::F32Lt,
        0x5e => Instruction::F32Gt,
        0x5f => Instruction::F32Le,
        0x60 => Instruction::F32Ge,
        0x61 => Instruction::F64Eq,
        0x62 => Instruction::F64Ne,
        0x63 => Instruction::F64Lt,
        0x64 => Instruction::F64Gt,
        0x65 => Instruction::F64Le,
        0x66 => Instruction::F64Ge,

        0x67 => Instruction::I32Clz,
        0x68 => Instruction::I32Ctz,
        0x69 => Instruction::I32Popcnt,
        0x6a => Instruction::I32Add,
        0x6b => Instruction::I32Sub,
        0x6c => Instruction::I32Mul,
        0x6d => Instruction::I32DivS,
        0x6e => Instruction::I32DivU,
        0x6f => Instruction::I32RemS,
        0x70 => Instruction::I32RemU,
        0x71 => Instruction::I32And,
        0x72 => Instruction::I32Or,
        0x73 => Instruction::I32Xor,
        0x74 => Instruction::I32Shl,
        0x75 => Instruction::I32ShrS,
        0x76 => Instruction::I32ShrU,
        0x77 => Instruction::I32Rotl,
        0x78 => Instruction::I32Rotr,
        0x79 => Instruction::I64Clz,
        0x7a => Instruction::I64Ctz,
        0x7b => Instruction::I64Popcnt,
        0x7c => Instruction::I64Add,
        0x7d => Instruction::I64Sub,
        0x7e => Instruction::I64Mul,
        0x7f => Instruction::I64DivS,
        0x80 => Instruction::I64DivU,
        0x81 => Instruction::I64RemS,
        0x82 => Instruction::I64RemU,
        0x83 => Instruction::I64And,
        0x84 => Instruction::I64Or,
        0x85 => Instruction::I64Xor,
        0x86 => Instruction::I64Shl,
        0x87 => Instruction::I64ShrS,
        0x88 => Instruction::I64ShrU,
        0x89 => Instruction::I64Rotl,
        0x8a => Instruction::I64Rotr,
        0x8b => Instruction::F32Abs,
        0x8c => Instruction::F32Neg,
        0x8d => Instruction::F32Ceil,
        0x8e => Instruction::F32Floor,
        0x8f => Instruction::F32Trunc,
        0x90 => Instruction::F32Nearest,
        0x91 => Instruction::F32Sqrt,
        0x92 => Instruction::F32Add,
        0x93 => Instruction::F32Sub,
        0x94 => Instruction::F32Mul,
        0x95 => Instruction::F32Div,
        0x96 => Instruction::F32Min,
        0x97 => Instruction::F32Max,
        0x98 => Instruction::F32CopySign,
        0x99 => Instruction::F64Abs,
        0x9a => Instruction::F64Neg,
        0x9b => Instruction::F64Ceil,
        0x9c => Instruction::F64Floor,
        0x9d => Instruction::F64Trunc,
        0x9e => Instruction::F64Nearest,
        0x9f => Instruction::F64Sqrt,
        0xa0 => Instruction::F64Add,
        0xa1 => Instruction::F64Sub,
        0xa2 => Instruction::F64Mul,
        0xa3 => Instruction::F64Div,
        0xa4 => Instruction::F64Min,
        0xa5 => Instruction::F64Max,
        0xa6 => Instruction::F64CopySign,

        0xa7 => Instruction::I32WrapI64,
        0xa8 => Instruction::I32TruncSF32,
        0xa9 => Instruction::I32TruncUF32,
        0xaa => Instruction::I32TruncSF64,
        0xab => Instruction::I32TruncUF64,
        0xac => Instruction::I64ExtendSI32,
        0xad => Instruction::I64ExtendUI32,
        0xae => Instruction::I64TruncSF32,
        0xaf => Instruction::I64TruncUF32,
        0xb0 => Instruction::I64TruncSF64,
        0xb1 => Instruction::I64TruncUF64,
        0xb2 => Instruction::F32ConvertSI32,
        0xb3 => Instruction::F32ConvertUI32,
        0xb4 => Instruction::F32ConvertSI64,
        0xb5 => Instruction::F32ConvertUI64,
        0xb6 => Instruction::F32DemoteF64,
        0xb7 => Instruction::F64ConvertSI32,
        0xb8 => Instruction::F64ConvertUI32,
        0xb9 => Instruction::F64ConvertSI64,
        0xba => Instruction::F64ConvertUI64,
        0xbb => Instruction::F64PromoteF32,

        0xbc => Instruction::I32ReinterpretF32,
        0xbd => Instruction::I64ReinterpretF64,
        0xbe => Instruction::F32ReinterpretI32,
        0xbf => Instruction::F64ReinterpretI64,

        _ => panic!("Invalid instruction"),
    }
}

/// https://webassembly.github.io/spec/core/binary/instructions.html#binary-expr
fn decode_expression(decoder: &mut Decoder) -> Expression {
    let mut instructions = Vec::new();

    while decoder.pick_byte().unwrap() != 0x0B {
        instructions.push(decode_instruction(decoder));
    }

    decoder.eat_byte(); // end

    instructions
}

fn decode_section<F>(decoder: &mut Decoder, section_id: u8, mut callback: F)
 where F: FnMut(&mut Decoder) {
     if decoder.pick_byte().unwrap() == section_id {
        decoder.eat_byte(); // section id
        
        let size = decode_u32(decoder);
        let offset = decoder.offset + size as usize;

        // Find a better way to avoid having to clone the bytes for each decoder.
        let mut closure_decoder = Decoder {
            bytes: decoder.bytes.clone(),
            offset: decoder.offset,
        };

        callback(&mut closure_decoder);

        if closure_decoder.offset != offset {
            panic!("Invalid section size");
        }

        decoder.offset = closure_decoder.offset;
    }
}

/// https://webassembly.github.io/spec/core/binary/modules.html#custom-section
fn decode_custom_sections(decoder: &mut Decoder) {
    while decoder.pick_byte().unwrap() == SECTION_ID_CUSTOM {
        decoder.eat_byte(); // section id

        // Consume the section content
        let section_size = decode_u32(decoder);
        for _i in 0..section_size {
            decoder.eat_byte();
        }
    }
}

/// https://webassembly.github.io/spec/core/binary/modules.html#type-section
fn decode_function_type_section(decoder: &mut Decoder) -> Vec<FunctionType> {
    let mut function_types = Vec::new();

    decode_section(decoder, SECTION_ID_TYPE, |decoder| {
        let vector_size = decode_u32(decoder);
        
        for _ in 0..vector_size {
            let function_type = decode_function_type(decoder);
            function_types.push(function_type);
        }
    });

    function_types
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-importsec
fn decode_import_section(decoder: &mut Decoder) -> Vec<Import> {
    let mut imports = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_IMPORT {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            imports.push(Import {
                module: decode_name(decoder),
                name: decode_name(decoder),
                descriptor: match decoder.eat_byte() {
                    0x00 => Index::Type(decode_u32(decoder)),
                    0x01 => Index::Table(decode_u32(decoder)),
                    0x02 => Index::Memory(decode_u32(decoder)),
                    0x03 => Index::Global(decode_u32(decoder)),
                    _ => panic!("Invalid import descriptor"),
                },
            })
        }
    }

    imports
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-funcsec
fn decode_function_section(decoder: &mut Decoder) -> Vec<Index> {
    let mut type_indexes = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_FUNCTION {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let type_index = Index::Type(decode_u32(decoder));
            type_indexes.push(type_index);
        }
    }

    type_indexes
}

/// https://webassembly.github.io/spec/core/binary/types.html#binary-tabletype
fn decode_table_type(decoder: &mut Decoder) -> TableType {
    let element_type = match decoder.eat_byte() {
        0x70 => ElementType::FuncRef,
        _ => panic!("Invalid element type"),
    };

    let limits = decode_limits(decoder);

    TableType {
        element_type: element_type,
        limits: limits,
    }
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-tablesec
fn decode_table_section(decoder: &mut Decoder) -> Vec<Table> {
    let mut tables = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_TABLE {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let table_type = decode_table_type(decoder);
            tables.push(Table {
                table_type: table_type,
            });
        }
    }

    tables
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-memsec
fn decode_memory_section(decoder: &mut Decoder) -> Vec<Memory> {
    let mut memories = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_MEMORY {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let limits = decode_limits(decoder);
            memories.push(Memory {
                memory_type: MemoryType { limits: limits },
            });
        }
    }

    memories
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-globalsec
fn decode_global_section(decoder: &mut Decoder) -> Vec<Global> {
    let mut globals = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_GLOBAL {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            globals.push(Global {
                global_type: decode_global_type(decoder),
                init: decode_expression(decoder),
            });
        }
    }

    globals
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-exportsec
fn decode_export_section(decoder: &mut Decoder) -> Vec<Export> {
    let mut exports = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_EXPORT {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            exports.push(Export {
                name: decode_name(decoder),
                descriptor: match decoder.eat_byte() {
                    0x00 => Index::Type(decode_u32(decoder)),
                    0x01 => Index::Table(decode_u32(decoder)),
                    0x02 => Index::Memory(decode_u32(decoder)),
                    0x03 => Index::Global(decode_u32(decoder)),
                    _ => panic!("Invalid export descriptor"),
                },
            })
        }
    }

    exports
}

/// https://webassembly.github.io/spec/core/binary/modules.html#start-section
fn decode_start_section(decoder: &mut Decoder) -> Option<StartFunction> {
    if decoder.pick_byte().unwrap() == SECTION_ID_START {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        Some(StartFunction {
            function: Index::Function(decode_u32(decoder)),
        })
    } else {
        None
    }
}

/// https://webassembly.github.io/spec/core/binary/modules.html#element-section
fn decode_element_section(decoder: &mut Decoder) -> Vec<Element> {
    let mut elements = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_ELEMENT {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let table = Index::Table(decode_u32(decoder));

            let offset = decode_expression(decoder);

            let mut init = Vec::new();
            let vector_size = decode_u32(decoder);

            for _ in 0..vector_size {
                init.push(Index::Function(decode_u32(decoder)));
            }

            elements.push(Element {
                table: table,
                offset: offset,
                init: init,
            });
        }
    }

    elements
}

/// https://webassembly.github.io/spec/core/binary/modules.html#code-section
fn decode_code_section(decoder: &mut Decoder) -> Vec<(Vec<ValueType>, Expression)> {
    let mut codes = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_CODE {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            decoder.eat_byte(); // code size

            let mut locals = Vec::new();
            let local_vector_size = decode_u32(decoder);

            for _ in 0..local_vector_size {
                let local_count = decode_u32(decoder);
                let value_type = decode_value_type(decoder);

                for _ in 0..local_count {
                    locals.push(value_type)
                }
            }

            let expression = decode_expression(decoder);

            codes.push((locals, expression))
        }
    }

    codes
}

/// https://webassembly.github.io/spec/core/binary/modules.html#data-section
fn decode_data_section(decoder: &mut Decoder) -> Vec<Data> {
    let mut datas = Vec::new();

    if decoder.pick_byte().unwrap() == SECTION_ID_DATA {
        decoder.eat_byte(); // section id
        decoder.eat_byte(); // section size

        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let data = Index::Memory(decode_u32(decoder));
            let offset = decode_expression(decoder);

            let mut init = Vec::new();
            let init_vector_size = decode_u32(decoder);

            for _ in 0..init_vector_size {
                init.push(decoder.eat_byte())
            }

            datas.push(Data {
                data: data,
                offset: offset,
                init: init,
            })
        }
    }

    datas
}

/// https://webassembly.github.io/spec/core/binary/modules.html
fn decode(bytes: Vec<u8>) -> Module {
    let mut decoder = Decoder {
        bytes: bytes,
        offset: 0,
    };

    assert!(
        decoder.eat_byte() == 0x00
            && decoder.eat_byte() == 0x61
            && decoder.eat_byte() == 0x73
            && decoder.eat_byte() == 0x6d,
        "Invalid magic string"
    );
    assert!(
        decoder.eat_byte() == 0x01
            && decoder.eat_byte() == 0x00
            && decoder.eat_byte() == 0x00
            && decoder.eat_byte() == 0x00,
        "Invalid version number"
    );

    println!("1");

    // TODO: Collect custom sections
    decode_custom_sections(&mut decoder);
    let function_types = decode_function_type_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let imports = decode_import_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let function_type_indexes = decode_function_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let tables = decode_table_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let memories = decode_memory_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let globals = decode_global_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let exports = decode_export_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let start = decode_start_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let elements = decode_element_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let codes = decode_code_section(&mut decoder);
    decode_custom_sections(&mut decoder);
    let data = decode_data_section(&mut decoder);
    decode_custom_sections(&mut decoder);

    if decoder.offset != decoder.bytes.len() {
        panic!("Unexpected end of file");
    }

    if function_type_indexes.len() != codes.len() {
        panic!("Function indexes and codes size mismatch");
    }

    let mut functions = Vec::new();

    for i in 0..function_type_indexes.len() {
        let type_index = function_type_indexes[i];
        let (locals, body) = &codes[i];

        // TODO: Understand if it's really necessary to clone the data structure here.
        functions.push(Function {
            function_type: type_index,
            locals: locals.clone(),
            body: body.clone(),
        })
    }

    Module {
        function_types: function_types,
        functions: functions,
        tables: tables,
        memories: memories,
        globals: globals,
        elements: elements,
        data: data,
        start: start,
        imports: imports,
        exports: exports,
    }
}
