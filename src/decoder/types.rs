use crate::decoder::values::decode_u32;
use crate::decoder::{Decoder, DecoderResult};
use crate::structure::*;

// https://webassembly.github.io/spec/core/binary/types.html#value-types
pub fn decode_value_type(decoder: &mut Decoder) -> DecoderResult<ValueType> {
    match decoder.eat_byte()? {
        0x7F => Ok(ValueType::I32),
        0x7E => Ok(ValueType::I64),
        0x7D => Ok(ValueType::F32),
        0x7C => Ok(ValueType::F64),
        _ => Err(decoder.produce_error("Invalid value type")),
    }
}

// https://webassembly.github.io/spec/core/binary/types.html#limits
pub fn decode_limits(decoder: &mut Decoder) -> DecoderResult<Limits> {
    match decoder.eat_byte()? {
        0x00 => Ok(Limits {
            min: decode_u32(decoder)?,
            max: None,
        }),
        0x01 => Ok(Limits {
            min: decode_u32(decoder)?,
            max: Some(decode_u32(decoder)?),
        }),
        _ => Err(decoder.produce_error("Invalid limit")),
    }
}

// https://webassembly.github.io/spec/core/binary/types.html#binary-functype
pub fn decode_function_type(decoder: &mut Decoder) -> DecoderResult<FunctionType> {
    if decoder.eat_byte()? != 0x60 {
        return Err(decoder.produce_error("Invalid function type prefix"));
    }

    let mut params = Vec::new();
    let mut results = Vec::new();

    let params_vector_size = decode_u32(decoder)?;
    for _ in 0..params_vector_size {
        let value_type = decode_value_type(decoder)?;
        params.push(value_type);
    }

    let results_vector_size = decode_u32(decoder)?;
    for _ in 0..results_vector_size {
        let value_type = decode_value_type(decoder)?;
        results.push(value_type);
    }

    Ok((params, results))
}

// https://webassembly.github.io/spec/core/binary/types.html#memory-types
pub fn decode_memory_type(decoder: &mut Decoder) -> DecoderResult<MemoryType> {
    Ok(MemoryType {
        limits: decode_limits(decoder)?,
    })
}

// https://webassembly.github.io/spec/core/binary/types.html#binary-globaltype
pub fn decode_global_type(decoder: &mut Decoder) -> DecoderResult<GlobalType> {
    Ok(GlobalType {
        value_type: decode_value_type(decoder)?,
        mutability: match decoder.eat_byte()? {
            0x00 => GlobalTypeMutability::Const,
            0x01 => GlobalTypeMutability::Var,
            _ => {
                return Err(decoder.produce_error("Invalid global type mutability"));
            }
        },
    })
}
