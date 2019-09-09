use crate::decoder::decoder::*;

fn decode_unsigned_leb_128(decoder: &mut Decoder) -> DecoderResult<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        let byte = u64::from(decoder.eat_byte()?);

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

    Ok(result)
}

pub fn decode_u32(decoder: &mut Decoder) -> DecoderResult<u32> {
    Ok(decode_unsigned_leb_128(decoder)? as u32)
}

pub fn decode_i32(decoder: &mut Decoder) -> DecoderResult<i32> {
    Ok(decode_unsigned_leb_128(decoder)? as i32)
}

pub fn decode_i64(decoder: &mut Decoder) -> DecoderResult<i64> {
    Ok(decode_unsigned_leb_128(decoder)? as i64)
}

pub fn decode_f32(decoder: &mut Decoder) -> DecoderResult<f32> {
    let mut bits: u32 = 0;

    for _ in 0..4 {
        bits = (bits << 8) | decoder.eat_byte()? as u32;
    }

    Ok(f32::from_bits(bits))
}

pub fn decode_f64(decoder: &mut Decoder) -> DecoderResult<f64> {
    let mut bits: u64 = 0;

    for _ in 0..8 {
        bits = (bits << 8) | decoder.eat_byte()? as u64;
    }

    Ok(f64::from_bits(bits))
}


/// https://webassembly.github.io/spec/core/binary/values.html#binary-name
///
/// The higher bits in the first byte contains a mask describing the number of byte encoding the
/// character. In UTF-8 characters can be encoded over 1 to 4 bytes.
pub fn decode_name(decoder: &mut Decoder) -> DecoderResult<String> {
    let mut chars = Vec::new();

    let vector_size = decode_u32(decoder)?;
    for _ in 0..vector_size {
        let byte1 = decoder.eat_byte()?;

        // 1 byte sequence with no continuation byte
        // [0xxxxxxx]
        if (byte1 & 0x80) == 0 {
            chars.push(byte1);
        }
        // 2 bytes sequence
        // [110xxxxx, 10xxxxxx]
        else if (byte1 & 0xe0) == 0xc0 {
            let byte2 = decoder.eat_byte()?;
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
            return Err(decoder.produce_error("Invalid utf-8 encoding"));
        }
    }

    match String::from_utf8(chars) {
        Ok(s) => Ok(s),
        Err(_) => Err(decoder.produce_error("Invalid utf-encoding")),
    }
}