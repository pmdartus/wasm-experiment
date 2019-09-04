use super::types::{
    BlockType, CustomSection, Data, Element, ElementType, Export, Expression, Function,
    FunctionType, Global, GlobalType, GlobalTypeMutability, Import, Index, Instruction, Limits,
    Memory, MemoryArg, MemoryType, Module, StartFunction, Table, TableType, ValueType,
};

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

#[derive(Debug, Copy, Clone)]
struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Decoder {
        Decoder { bytes, offset: 0 }
    }

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

    fn match_byte(&mut self, expected: u8) -> bool {
        match self.pick_byte() {
            Some(actual) if actual == expected => {
                self.offset += 1;
                true
            }
            _ => false,
        }
    }
}

fn decode_unsigned_leb_128(decoder: &mut Decoder) -> u64 {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        let byte = u64::from(decoder.eat_byte());

        // Extract the low order 7 bits of byte, left shift the byte and add them to the current
        // result.
        result |= (byte & 0x7f) << (shift * 7);

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

fn decode_f32(_decoder: &mut Decoder) -> f32 {
    panic!("Not implemented")
}

fn decode_f64(_decoder: &mut Decoder) -> f64 {
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
        0x24 => Instruction::GlobalSet(Index::Global(decode_u32(decoder))),

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
        0x3f => Instruction::MemorySize,
        0x40 => Instruction::MemoryGrow,

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
where
    F: FnMut(&mut Decoder),
{
    if decoder.match_byte(section_id) {
        let size = decode_u32(decoder);
        let end_offset = decoder.offset + size as usize;

        let closure_decoder = &mut decoder.clone();
        callback(closure_decoder);

        if closure_decoder.offset != end_offset {
            panic!("Invalid section size");
        }

        decoder.offset = closure_decoder.offset;
    }
}

/// https://webassembly.github.io/spec/core/binary/modules.html#custom-section
fn decode_custom_sections<'a>(
    decoder: &mut Decoder<'a>,
    custom_sections: &mut Vec<CustomSection<'a>>,
) {
    while decoder.match_byte(SECTION_ID_CUSTOM) {
        let size = decode_u32(decoder);
        let end_offset = decoder.offset + size as usize;

        let name = decode_name(decoder);
        let bytes = &decoder.bytes[decoder.offset..end_offset];

        custom_sections.push((name, bytes));
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

    decode_section(decoder, SECTION_ID_IMPORT, |decoder| {
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
    });

    imports
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-funcsec
fn decode_function_section(decoder: &mut Decoder) -> Vec<Index> {
    let mut type_indexes = Vec::new();

    decode_section(decoder, SECTION_ID_FUNCTION, |decoder| {
        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let type_index = Index::Type(decode_u32(decoder));
            type_indexes.push(type_index);
        }
    });

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
        element_type,
        limits,
    }
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-tablesec
fn decode_table_section(decoder: &mut Decoder) -> Vec<Table> {
    let mut tables = Vec::new();

    decode_section(decoder, SECTION_ID_TABLE, |decoder| {
        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let table_type = decode_table_type(decoder);
            tables.push(Table { table_type });
        }
    });

    tables
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-memsec
fn decode_memory_section(decoder: &mut Decoder) -> Vec<Memory> {
    let mut memories = Vec::new();

    decode_section(decoder, SECTION_ID_MEMORY, |decoder| {
        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let limits = decode_limits(decoder);
            let memory_type = MemoryType { limits };
            memories.push(Memory { memory_type });
        }
    });

    memories
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-globalsec
fn decode_global_section(decoder: &mut Decoder) -> Vec<Global> {
    let mut globals = Vec::new();

    decode_section(decoder, SECTION_ID_GLOBAL, |decoder| {
        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            globals.push(Global {
                global_type: decode_global_type(decoder),
                init: decode_expression(decoder),
            });
        }
    });

    globals
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-exportsec
fn decode_export_section(decoder: &mut Decoder) -> Vec<Export> {
    let mut exports = Vec::new();

    decode_section(decoder, SECTION_ID_EXPORT, |decoder| {
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
    });

    exports
}

/// https://webassembly.github.io/spec/core/binary/modules.html#start-section
fn decode_start_section(decoder: &mut Decoder) -> Option<StartFunction> {
    if decoder.match_byte(SECTION_ID_START) {
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

    decode_section(decoder, SECTION_ID_ELEMENT, |decoder| {
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
                table,
                offset,
                init,
            });
        }
    });

    elements
}

/// https://webassembly.github.io/spec/core/binary/modules.html#code-section
fn decode_code_section(decoder: &mut Decoder) -> Vec<(Vec<ValueType>, Expression)> {
    let mut codes = Vec::new();

    decode_section(decoder, SECTION_ID_CODE, |decoder| {
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
    });

    codes
}

/// https://webassembly.github.io/spec/core/binary/modules.html#data-section
fn decode_data_section(decoder: &mut Decoder) -> Vec<Data> {
    let mut datas = Vec::new();

    decode_section(decoder, SECTION_ID_DATA, |decoder| {
        let vector_size = decode_u32(decoder);
        for _ in 0..vector_size {
            let data = Index::Memory(decode_u32(decoder));
            let offset = decode_expression(decoder);

            let mut init = Vec::new();
            let init_vector_size = decode_u32(decoder);

            for _ in 0..init_vector_size {
                init.push(decoder.eat_byte())
            }

            datas.push(Data { data, offset, init })
        }
    });

    datas
}

/// https://webassembly.github.io/spec/core/binary/modules.html
pub fn decode(bytes: &[u8]) -> Module {
    let decoder = &mut Decoder::new(bytes);
    let mut custom_sections = Vec::new();

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

    decode_custom_sections(decoder, &mut custom_sections);
    let function_types = decode_function_type_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let imports = decode_import_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let function_type_indexes = decode_function_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let tables = decode_table_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let memories = decode_memory_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let globals = decode_global_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let exports = decode_export_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let start = decode_start_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let elements = decode_element_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let codes = decode_code_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);
    let data = decode_data_section(decoder);
    decode_custom_sections(decoder, &mut custom_sections);

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
        custom_sections,
        function_types,
        functions,
        tables,
        memories,
        globals,
        elements,
        data,
        start,
        imports,
        exports,
    }
}