use crate::structure::*;
use crate::validation::instructions::{validate_constant_expression, validate_expression};
use crate::validation::types::{
    validate_function_type, validate_global_type, validate_memory_type, validate_table_type,
};
use crate::validation::validation::{Context, ValidationError, ValidationResult};

// https://webassembly.github.io/spec/core/valid/modules.html#valid-func
fn validate_function(context: &Context, function: &Function) -> ValidationResult {
    let function_type = context.get_function_type(function.function_type)?;

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
    context.get_table(element.table)?;

    // No need to validate the table type, since there is only one type of element types in table
    // right now.

    validate_expression(context, &element.offset, vec![ValueType::I32])?;
    validate_constant_expression(context, &element.offset)?;

    for init in &element.init {
        context.get_function(*init)?;
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-data
fn validate_data(context: &Context, data: &Data) -> ValidationResult {
    context.get_memory(data.data)?;

    validate_expression(context, &data.offset, vec![ValueType::I32])?;
    validate_constant_expression(context, &data.offset)?;

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-start
fn validate_start(context: &Context, start: &StartFunction) -> ValidationResult {
    let function = context.get_function(start.function)?;

    // Function type have been validated previously, we should not worry to unwrap the value.
    let (params, returns) = context.get_function_type(function.function_type)?;
    if !params.is_empty() || !returns.is_empty() {
        return Err(ValidationError::from("Invalid start function"));
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/modules.html#valid-import
fn validate_import(context: &Context, import: &Import) -> ValidationResult {
    match &import.descriptor {
        ImportDescriptor::Function(function) => {
            context.get_function(*function)?;
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
            context.get_function(*function)?;
        }
        ExportDescriptor::Table(table) => {
            context.get_table(*table)?;
        }
        ExportDescriptor::Memory(memory) => {
            context.get_memory(*memory)?;
        }
        ExportDescriptor::Global(global) => {
            context.get_global(*global)?;
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
