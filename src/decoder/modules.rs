use crate::decoder::instructions::decode_expression;
use crate::decoder::types::{
    decode_function_type, decode_global_type, decode_limits, decode_memory_type, decode_value_type,
};
use crate::decoder::values::{decode_name, decode_u32};
use crate::decoder::{Decoder, DecoderResult};
use crate::structure::*;

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

fn decode_section<F, R>(decoder: &mut Decoder, section_id: u8, mut callback: F) -> DecoderResult<()>
where
    F: FnMut(&mut Decoder) -> DecoderResult<R>,
{
    if decoder.match_byte(section_id) {
        let size = decode_u32(decoder)?;
        let end_offset = decoder.offset + size as usize;

        let closure_decoder = &mut decoder.clone();
        callback(closure_decoder)?;

        if closure_decoder.offset != end_offset {
            return Err(closure_decoder.produce_error("Invalid section size"));
        }

        decoder.offset = closure_decoder.offset;
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/binary/modules.html#custom-section
fn decode_custom_sections<'a>(
    decoder: &mut Decoder<'a>,
    custom_sections: &mut Vec<CustomSection<'a>>,
) -> DecoderResult<()> {
    while decoder.match_byte(SECTION_ID_CUSTOM) {
        let size = decode_u32(decoder)?;
        let end_offset = decoder.offset + size as usize;

        let name = decode_name(decoder)?;

        // Before creating a new slice we need to make sure that the custom section bytes slice is
        // within the boundary of the original slice and also that the current offset is not
        // greater than the section size.
        if decoder.offset > decoder.bytes.len()
            || end_offset > decoder.bytes.len()
            || decoder.offset > end_offset
        {
            return Err(decoder.produce_error("Invalid section size"));
        }

        let bytes = &decoder.bytes[decoder.offset..end_offset];

        custom_sections.push((name, bytes));
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/binary/modules.html#type-section
fn decode_function_type_section(decoder: &mut Decoder) -> DecoderResult<Vec<FunctionType>> {
    let mut function_types = Vec::new();

    decode_section(decoder, SECTION_ID_TYPE, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            let function_type = decode_function_type(decoder)?;
            function_types.push(function_type);
        }
        Ok(())
    })?;

    Ok(function_types)
}

// https://webassembly.github.io/spec/core/binary/modules.html#binary-importsec
fn decode_import_section(decoder: &mut Decoder) -> DecoderResult<Vec<Import>> {
    let mut imports = Vec::new();

    decode_section(decoder, SECTION_ID_IMPORT, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            imports.push(Import {
                module: decode_name(decoder)?,
                name: decode_name(decoder)?,
                descriptor: match decoder.eat_byte()? {
                    0x00 => ImportDescriptor::Function(decode_u32(decoder)?),
                    0x01 => ImportDescriptor::Table(decode_table_type(decoder)?),
                    0x02 => ImportDescriptor::Memory(decode_memory_type(decoder)?),
                    0x03 => ImportDescriptor::Global(decode_global_type(decoder)?),
                    _ => return Err(decoder.produce_error("Invalid import descriptor")),
                },
            })
        }
        Ok(())
    })?;

    Ok(imports)
}

// https://webassembly.github.io/spec/core/binary/modules.html#binary-funcsec
fn decode_function_section(decoder: &mut Decoder) -> DecoderResult<Vec<u32>> {
    let mut type_indexes = Vec::new();

    decode_section(decoder, SECTION_ID_FUNCTION, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            let type_index = decode_u32(decoder)?;
            type_indexes.push(type_index);
        }
        Ok(())
    })?;

    Ok(type_indexes)
}

// https://webassembly.github.io/spec/core/binary/types.html#binary-tabletype
fn decode_table_type(decoder: &mut Decoder) -> DecoderResult<TableType> {
    let element_type = match decoder.eat_byte()? {
        0x70 => ElementType::FuncRef,
        _ => return Err(decoder.produce_error("Invalid element type")),
    };

    let limits = decode_limits(decoder)?;

    Ok(TableType {
        element_type,
        limits,
    })
}

// https://webassembly.github.io/spec/core/binary/modules.html#binary-tablesec
fn decode_table_section(decoder: &mut Decoder) -> DecoderResult<Vec<Table>> {
    let mut tables = Vec::new();

    decode_section(decoder, SECTION_ID_TABLE, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            let table_type = decode_table_type(decoder)?;
            tables.push(Table { table_type });
        }
        Ok(())
    })?;

    Ok(tables)
}

// https://webassembly.github.io/spec/core/binary/modules.html#binary-memsec
fn decode_memory_section(decoder: &mut Decoder) -> DecoderResult<Vec<Memory>> {
    let mut memories = Vec::new();

    decode_section(decoder, SECTION_ID_MEMORY, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            memories.push(Memory {
                memory_type: decode_memory_type(decoder)?,
            });
        }
        Ok(())
    })?;

    Ok(memories)
}

// https://webassembly.github.io/spec/core/binary/modules.html#binary-globalsec
fn decode_global_section(decoder: &mut Decoder) -> DecoderResult<Vec<Global>> {
    let mut globals = Vec::new();

    decode_section(decoder, SECTION_ID_GLOBAL, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            globals.push(Global {
                global_type: decode_global_type(decoder)?,
                init: decode_expression(decoder)?,
            });
        }
        Ok(())
    })?;

    Ok(globals)
}

// https://webassembly.github.io/spec/core/binary/modules.html#binary-exportsec
fn decode_export_section(decoder: &mut Decoder) -> DecoderResult<Vec<Export>> {
    let mut exports = Vec::new();

    decode_section(decoder, SECTION_ID_EXPORT, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            exports.push(Export {
                name: decode_name(decoder)?,
                descriptor: match decoder.eat_byte()? {
                    0x00 => ExportDescriptor::Function(decode_u32(decoder)?),
                    0x01 => ExportDescriptor::Table(decode_u32(decoder)?),
                    0x02 => ExportDescriptor::Memory(decode_u32(decoder)?),
                    0x03 => ExportDescriptor::Global(decode_u32(decoder)?),
                    _ => return Err(decoder.produce_error("Invalid export descriptor")),
                },
            })
        }
        Ok(())
    })?;

    Ok(exports)
}

// https://webassembly.github.io/spec/core/binary/modules.html#start-section
fn decode_start_section(decoder: &mut Decoder) -> DecoderResult<Option<StartFunction>> {
    if decoder.match_byte(SECTION_ID_START) {
        decoder.eat_byte()?; // section size

        Ok(Some(StartFunction {
            function: decode_u32(decoder)?,
        }))
    } else {
        Ok(None)
    }
}

// https://webassembly.github.io/spec/core/binary/modules.html#element-section
fn decode_element_section(decoder: &mut Decoder) -> DecoderResult<Vec<Element>> {
    let mut elements = Vec::new();

    decode_section(decoder, SECTION_ID_ELEMENT, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            let table = decode_u32(decoder)?;

            let offset = decode_expression(decoder)?;

            let mut init = Vec::new();
            let vector_size = decode_u32(decoder)?;

            for _ in 0..vector_size {
                init.push(decode_u32(decoder)?);
            }

            elements.push(Element {
                table,
                offset,
                init,
            });
        }
        Ok(())
    })?;

    Ok(elements)
}

// https://webassembly.github.io/spec/core/binary/modules.html#code-section
fn decode_code_section(
    decoder: &mut Decoder,
) -> DecoderResult<Vec<(Vec<(u32, ValueType)>, Expression)>> {
    let mut codes = Vec::new();

    decode_section(decoder, SECTION_ID_CODE, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            let code_size = decode_u32(decoder)?;
            let end_offset = decoder.offset + code_size as usize;

            let mut locals = Vec::new();
            let mut total_local_count: u64 = 0;

            let local_vector_size = decode_u32(decoder)?;

            for _ in 0..local_vector_size {
                let local_count = decode_u32(decoder)?;
                let value_type = decode_value_type(decoder)?;

                total_local_count += local_count as u64;

                locals.push((local_count, value_type));
            }

            let base: u64 = 2;
            if total_local_count > base.pow(32) {
                return Err(decoder.produce_error("Too many locals"));
            }

            let expression = decode_expression(decoder)?;

            if decoder.offset != end_offset {
                return Err(decoder.produce_error("Invalid code size"));
            }

            codes.push((locals, expression))
        }
        Ok(())
    })?;

    Ok(codes)
}

// https://webassembly.github.io/spec/core/binary/modules.html#data-section
fn decode_data_section(decoder: &mut Decoder) -> DecoderResult<Vec<Data>> {
    let mut datas = Vec::new();

    decode_section(decoder, SECTION_ID_DATA, |decoder| {
        let vector_size = decode_u32(decoder)?;
        for _ in 0..vector_size {
            let data = decode_u32(decoder)?;
            let offset = decode_expression(decoder)?;

            let mut init = Vec::new();
            let init_vector_size = decode_u32(decoder)?;

            for _ in 0..init_vector_size {
                init.push(decoder.eat_byte()?)
            }

            datas.push(Data { data, offset, init })
        }
        Ok(())
    })?;

    Ok(datas)
}

// https://webassembly.github.io/spec/core/binary/modules.html
pub fn decode(bytes: &[u8]) -> DecoderResult<Module> {
    let decoder = &mut Decoder::new(bytes);
    let mut custom_sections = Vec::new();

    if decoder.eat_byte()? != 0x00
        || decoder.eat_byte()? != 0x61
        || decoder.eat_byte()? != 0x73
        || decoder.eat_byte()? != 0x6d
    {
        return Err(decoder.produce_error("Invalid magic string"));
    }
    if decoder.eat_byte()? != 0x01
        || decoder.eat_byte()? != 0x00
        || decoder.eat_byte()? != 0x00
        || decoder.eat_byte()? != 0x00
    {
        return Err(decoder.produce_error("Invalid version number"));
    }

    decode_custom_sections(decoder, &mut custom_sections)?;
    let function_types = decode_function_type_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let imports = decode_import_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let function_type_indexes = decode_function_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let tables = decode_table_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let memories = decode_memory_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let globals = decode_global_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let exports = decode_export_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let start = decode_start_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let elements = decode_element_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let codes = decode_code_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;
    let data = decode_data_section(decoder)?;
    decode_custom_sections(decoder, &mut custom_sections)?;

    if decoder.offset != decoder.bytes.len() {
        return Err(decoder.produce_error("Unexpected end of file"));
    }

    if function_type_indexes.len() != codes.len() {
        return Err(decoder.produce_error("Function indexes and codes size mismatch"));
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

    Ok(Module {
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
    })
}
