use std::{u16, u32};

use crate::structure::*;
use crate::validation::validation::{ValidationError, ValidationResult};

// https://webassembly.github.io/spec/core/valid/types.html#limits
pub fn validate_limits(limits: &Limits, range: u32) -> ValidationResult {
    if limits.min > range {
        return Err(ValidationError::from("Limit minimum is above valid range"));
    }

    match limits.max {
        Some(max) => {
            if max > range {
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
pub fn validate_function_type(function_type: &FunctionType) -> ValidationResult {
    let (_params, returns) = function_type;

    if returns.len() > 1 {
        return Err(ValidationError::from("Too many return value"));
    }

    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-tabletype
pub fn validate_table_type(table_type: &TableType) -> ValidationResult {
    validate_limits(&table_type.limits, u32::MAX)?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-memtype
pub fn validate_memory_type(memory_type: &MemoryType) -> ValidationResult {
    validate_limits(&memory_type.limits, u16::MAX as u32)?;
    Ok(())
}

// https://webassembly.github.io/spec/core/valid/types.html#valid-globaltype
pub fn validate_global_type(_global_type: &GlobalType) -> ValidationResult {
    Ok(())
}