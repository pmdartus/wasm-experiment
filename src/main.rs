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

enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

type FunctionType = (Vec<ValueType>, Vec<ValueType>);

struct Limits {
    min: u32,
    max: Option<u32>,
}

enum ElementType {
    FuncRef,
}

struct MemoryType {
    limits: Limits,
}

struct TableType {
    limits: Limits,
    element_type: ElementType,
}

enum GlobalTypeMutability {
    Const,
    Var,
}

struct GlobalType {
    value_type: ValueType,
    mutability: GlobalTypeMutability,
}

struct Table {
    table_type: TableType,
}

struct Memory {
    memory_type: MemoryType,
}

struct Global {
    global_type: GlobalType,
    // TODO: expression:
}

struct Decoder {
    bytes: Vec<u8>,
    offset: usize,
}

impl Decoder {
    fn eat_byte(&mut self) -> u8 {
        let ret = self.pick_byte();
        self.offset += 1;
        ret
    }

    fn pick_byte(&self) -> u8 {
        self.bytes[self.offset]
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
        0x7F => ValueType::F32,
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

/// https://webassembly.github.io/spec/core/binary/modules.html#custom-section
fn decode_custom_sections(decoder: &mut Decoder) {
    while decoder.pick_byte() == SECTION_ID_CUSTOM {
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
    decoder.eat_byte(); // section id
    decoder.eat_byte(); // section size

    let mut function_types = Vec::new();

    let vector_size = decode_u32(decoder);
    for _ in 0..vector_size {
        let function_type = decode_function_type(decoder);
        function_types.push(function_type);
    }

    function_types
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-importsec
fn decode_import_section() {
    panic!("Not implemented");
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-funcsec
fn decode_function_section(decoder: &mut Decoder) -> Vec<u32> {
    decoder.eat_byte(); // section id
    decoder.eat_byte(); // section size

    let mut type_indexes = Vec::new();

    let vector_size = decode_u32(decoder);
    for _ in 0..vector_size {
        let type_index = decode_u32(decoder);
        type_indexes.push(type_index);
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
    decoder.eat_byte(); // section id
    decoder.eat_byte(); // section size

    let mut tables = Vec::new();

    let vector_size = decode_u32(decoder);
    for _ in 0..vector_size {
        let table_type = decode_table_type(decoder);
        tables.push(Table {
            table_type: table_type,
        });
    }

    tables
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-memsec
fn decode_memory_section(decoder: &mut Decoder) -> Vec<Memory> {
    decoder.eat_byte(); // section id
    decoder.eat_byte(); // section size

    let mut memories = Vec::new();

    let vector_size = decode_u32(decoder);
    for _ in 0..vector_size {
        let limits = decode_limits(decoder);
        memories.push(Memory {
            memory_type: MemoryType { limits: limits },
        });
    }

    memories
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-globalsec
fn decode_global_section(decoder: &mut Decoder) -> Vec<Global> {
    decoder.eat_byte(); // section id
    decoder.eat_byte(); // section size

    let mut globals = Vec::new();

    let vector_size = decode_u32(decoder);
    for _ in 0..vector_size {
        let global_type = decode_global_type(decoder);
        globals.push(Global {
            global_type: global_type,
        });
    }

    globals
}

/// https://webassembly.github.io/spec/core/binary/modules.html#binary-exportsec
fn decode_export_section(decoder: &mut Decoder) {
    panic!("Not implemented");
}

/// https://webassembly.github.io/spec/core/binary/modules.html#start-section
fn decode_start_section(decoder: &mut Decoder) {
    panic!("Not implemented");
}

/// https://webassembly.github.io/spec/core/binary/modules.html#element-section
fn decode_element_section(decoder: &mut Decoder) {
    panic!("Not implemented");
}

/// https://webassembly.github.io/spec/core/binary/modules.html#code-section
fn decode_code_section(decoder: &mut Decoder) {
    panic!("Not implemented");
}

/// https://webassembly.github.io/spec/core/binary/modules.html#data-section
fn decode_data_section(decoder: &mut Decoder) {
    panic!("Not implemented");
}

/// https://webassembly.github.io/spec/core/binary/modules.html
fn decode(bytes: Vec<u8>) -> () {
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

    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_TYPE {
        decode_function_type_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_IMPORT {
        decode_import_section();
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_FUNCTION {
        decode_function_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_TABLE {
        decode_table_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_MEMORY {
        decode_memory_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_GLOBAL {
        decode_global_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_EXPORT {
        decode_export_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_START {
        decode_start_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_ELEMENT {
        decode_element_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CODE {
        decode_code_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_DATA {
        decode_data_section(&mut decoder);
    }
    if decoder.pick_byte() == SECTION_ID_CUSTOM {
        decode_custom_sections(&mut decoder);
    }
}
