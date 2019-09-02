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

enum ValueType {
    I32,
    I64,
    F32,
    F64
}

type FunctionType = (Vec<ValueType>, Vec<ValueType>);

struct Decoder {
    bytes: Vec<u8>,
    offset: usize,
}

impl Decoder {
    fn eatByte(&mut self) -> u8 {
        let ret = self.pickByte();
        self.offset += 1;
        ret
    }

    fn pickByte(&self) -> u8 {
        self.bytes[self.offset]
    }
}

fn decode_unsigned_leb_128(mut decoder: &Decoder) -> u64 {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        let byte = decoder.eatByte() as u64;

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

/// https://webassembly.github.io/spec/core/binary/types.html#value-types
fn decode_value_type(mut decoder: &Decoder) -> ValueType {
    match decoder.eatByte() {
        0x7F => ValueType::F32,
        0x7E => ValueType::I64,
        0x7D => ValueType::F32,
        0x7C => ValueType::F64,
        _ => panic!("Invalid value type")
    }
}

/// https://webassembly.github.io/spec/core/binary/types.html#binary-functype
fn decode_function_type(mut decoder: &Decoder) -> FunctionType {
    assert!(decoder.eatByte() == 0x60, "Invalid function type prefix");

    let mut params = Vec::new();
    let mut results = Vec::new();

    let params_vector_size = decode_unsigned_leb_128(decoder);
    for _ in 0..params_vector_size {
        let value_type = decode_value_type(decoder);
        params.push(value_type);
    }

    let results_vector_size = decode_unsigned_leb_128(decoder);
    for _ in 0..results_vector_size {
        let value_type = decode_value_type(decoder);
        results.push(value_type);
    }

    (params, results)
}

/// https://webassembly.github.io/spec/core/binary/modules.html#custom-section
fn decode_custom_section() {
    panic!("Missing implementation");
}

/// https://webassembly.github.io/spec/core/binary/modules.html#type-section
fn decode_type_section(mut decoder: &Decoder) -> Vec<FunctionType> {
    decoder.eatByte(); // section id
    decoder.eatByte(); // section size

    let mut function_types = Vec::new();

    let vector_size = decode_unsigned_leb_128(decoder);
    for _ in 0..vector_size {
        let function_type = decode_function_type(decoder);
        function_types.push(function_type);
    }

    function_types
}

/// https://webassembly.github.io/spec/core/binary/modules.html
fn decode(bytes: Vec<u8>) -> () {
    let mut decoder = Decoder { bytes: bytes, offset: 0 };

    assert!(
        decoder.eatByte() == 0x00
            && decoder.eatByte() == 0x61
            && decoder.eatByte() == 0x73
            && decoder.eatByte() == 0x6d,
        "Invalid magic string"
    );
    assert!(
        decoder.eatByte() == 0x01
            && decoder.eatByte() == 0x00
            && decoder.eatByte() == 0x00
            && decoder.eatByte() == 0x00,
        "Invalid version number"
    );

    if decoder.pickByte() == SECTION_ID_CUSTOM {
        decode_custom_section();
    } else if decoder.pickByte() == SECTION_ID_TYPE {
        decode_type_section(&decoder);
    }
}