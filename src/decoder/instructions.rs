use crate::decoder::decoder::{Decoder, DecoderResult};
use crate::decoder::types::decode_value_type;
use crate::decoder::values::{decode_f32, decode_f64, decode_i32, decode_i64, decode_u32};
use crate::structure::*;

// https://webassembly.github.io/spec/core/binary/types.html#binary-blocktype
fn decode_block_type(decoder: &mut Decoder) -> DecoderResult<BlockType> {
    Ok(if decoder.match_byte(0x40) {
        BlockType::Void
    } else {
        BlockType::Return(decode_value_type(decoder)?)
    })
}

// https://webassembly.github.io/spec/core/binary/instructions.html#binary-memarg
fn decode_memory_arg(decoder: &mut Decoder) -> DecoderResult<MemoryArg> {
    Ok(MemoryArg {
        align: decode_u32(decoder)?,
        offset: decode_u32(decoder)?,
    })
}

// https://webassembly.github.io/spec/core/binary/instructions.html#instructions
fn decode_instruction(decoder: &mut Decoder) -> DecoderResult<Instruction> {
    Ok(match decoder.eat_byte()? {
        0x00 => Instruction::Unreachable,
        0x01 => Instruction::Nop,
        0x02 => {
            let block_type = decode_block_type(decoder)?;

            let mut instructions = Vec::new();
            while decoder.pick_byte().unwrap() != 0x0B {
                instructions.push(decode_instruction(decoder)?);
            }

            decoder.eat_byte()?; // end
            Instruction::Block(block_type, instructions)
        }
        0x03 => {
            let block_type = decode_block_type(decoder)?;

            let mut instructions = Vec::new();
            while decoder.pick_byte().unwrap() != 0x0B {
                instructions.push(decode_instruction(decoder)?);
            }

            decoder.eat_byte()?; // end
            Instruction::Loop(block_type, instructions)
        }
        0x04 => {
            let block_type = decode_block_type(decoder)?;

            let mut if_instructions = Vec::new();
            while decoder.pick_byte().unwrap() != 0x0B && decoder.pick_byte().unwrap() != 0x05 {
                if_instructions.push(decode_instruction(decoder)?);
            }

            let else_instructions = if decoder.match_byte(0x05) {
                let mut else_instructions = Vec::new();

                while decoder.pick_byte().unwrap() != 0x0B {
                    else_instructions.push(decode_instruction(decoder)?);
                }

                Some(else_instructions)
            } else {
                None
            };

            decoder.eat_byte()?; // end
            Instruction::If(block_type, if_instructions, else_instructions)
        }
        0x0C => Instruction::Br(decode_u32(decoder)?),
        0x0D => Instruction::BrIf(decode_u32(decoder)?),
        0x0E => {
            let mut labels = Vec::new();

            let vector_size = decode_u32(decoder)?;
            for _ in 0..vector_size {
                labels.push(decode_u32(decoder)?);
            }

            let default_label = decode_u32(decoder)?;

            Instruction::BrTable(labels, default_label)
        }
        0x0F => Instruction::Return,
        0x10 => Instruction::Call(decode_u32(decoder)?),
        0x11 => {
            let index = decode_u32(decoder)?;
            if !decoder.match_byte(0x00) {
                return Err(
                    decoder.produce_error("Invalid reserved byte after call_indirect instruction")
                );
            }

            Instruction::CallIndirect(index)
        }

        0x1A => Instruction::Drop,
        0x1B => Instruction::Select,

        0x20 => Instruction::LocalGet(decode_u32(decoder)?),
        0x21 => Instruction::LocalSet(decode_u32(decoder)?),
        0x22 => Instruction::LocalTee(decode_u32(decoder)?),
        0x23 => Instruction::GlobalGet(decode_u32(decoder)?),
        0x24 => Instruction::GlobalSet(decode_u32(decoder)?),

        0x28 => Instruction::I32Load(decode_memory_arg(decoder)?),
        0x29 => Instruction::I64Load(decode_memory_arg(decoder)?),
        0x2a => Instruction::F32Load(decode_memory_arg(decoder)?),
        0x2b => Instruction::F64Load(decode_memory_arg(decoder)?),
        0x2c => Instruction::I32Load8S(decode_memory_arg(decoder)?),
        0x2d => Instruction::I32Load8U(decode_memory_arg(decoder)?),
        0x2e => Instruction::I32Load16S(decode_memory_arg(decoder)?),
        0x2f => Instruction::I32Load16U(decode_memory_arg(decoder)?),
        0x30 => Instruction::I64Load8S(decode_memory_arg(decoder)?),
        0x31 => Instruction::I64Load8U(decode_memory_arg(decoder)?),
        0x32 => Instruction::I64Load16S(decode_memory_arg(decoder)?),
        0x33 => Instruction::I64Load16U(decode_memory_arg(decoder)?),
        0x34 => Instruction::I64Load32S(decode_memory_arg(decoder)?),
        0x35 => Instruction::I64Load32U(decode_memory_arg(decoder)?),
        0x36 => Instruction::I32Store(decode_memory_arg(decoder)?),
        0x37 => Instruction::I64Store(decode_memory_arg(decoder)?),
        0x38 => Instruction::F32Store(decode_memory_arg(decoder)?),
        0x39 => Instruction::F64Store(decode_memory_arg(decoder)?),
        0x3a => Instruction::I32Store8(decode_memory_arg(decoder)?),
        0x3b => Instruction::I32Store16(decode_memory_arg(decoder)?),
        0x3c => Instruction::I64Store8(decode_memory_arg(decoder)?),
        0x3d => Instruction::I64Store16(decode_memory_arg(decoder)?),
        0x3e => Instruction::I64Store32(decode_memory_arg(decoder)?),
        0x3f => {
            if !decoder.match_byte(0x00) {
                return Err(
                    decoder.produce_error("Invalid reserved byte after memory_size instruction")
                );
            }

            Instruction::MemorySize
        }
        0x40 => {
            if !decoder.match_byte(0x00) {
                return Err(
                    decoder.produce_error("Invalid reserved byte after memory_grow instruction")
                );
            }

            Instruction::MemoryGrow
        }

        0x41 => Instruction::I32Const(decode_i32(decoder)?),
        0x42 => Instruction::I64Const(decode_i64(decoder)?),
        0x43 => Instruction::F32Const(decode_f32(decoder)?),
        0x44 => Instruction::F64Const(decode_f64(decoder)?),

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

        _ => return Err(decoder.produce_error("Invalid instruction")),
    })
}

// https://webassembly.github.io/spec/core/binary/instructions.html#binary-expr
pub fn decode_expression(decoder: &mut Decoder) -> DecoderResult<Expression> {
    let mut instructions = Vec::new();

    while decoder.pick_byte() != Some(0x0B) {
        instructions.push(decode_instruction(decoder)?);
    }

    decoder.eat_byte()?; // end

    Ok(instructions)
}
