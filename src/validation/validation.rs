use crate::structure::*;

#[derive(Debug)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn from(message: &str) -> ValidationError {
        ValidationError {
            message: String::from(message),
        }
    }

    pub fn from_string(message: String) -> ValidationError {
        ValidationError { message }
    }
}

pub type ValidationResult = Result<(), ValidationError>;

#[derive(Debug)]
pub struct Context<'a> {
    pub function_types: &'a Vec<FunctionType>,
    pub functions: &'a Vec<Function>,
    pub tables: &'a Vec<Table>,
    pub memories: &'a Vec<Memory>,
    pub globals: &'a Vec<Global>,
    pub elements: &'a Vec<Element>,
    pub locals: Vec<ValueType>,
}

impl<'a> Context<'a> {
    pub fn get_function_type(
        &self,
        function_type_index: u32,
    ) -> Result<&'a FunctionType, ValidationError> {
        self.function_types
            .get(function_type_index as usize)
            .ok_or(ValidationError::from("Invalid function type reference"))
    }

    pub fn get_function(&self, function_index: u32) -> Result<&'a Function, ValidationError> {
        self.functions
            .get(function_index as usize)
            .ok_or(ValidationError::from("Invalid function reference"))
    }

    pub fn get_table(&self, table_index: u32) -> Result<&'a Table, ValidationError> {
        self.tables
            .get(table_index as usize)
            .ok_or(ValidationError::from("Invalid table reference"))
    }

    pub fn get_memory(&self, memory_index: u32) -> Result<&'a Memory, ValidationError> {
        self.memories
            .get(memory_index as usize)
            .ok_or(ValidationError::from("Invalid memory reference"))
    }

    pub fn get_global(&self, global_index: u32) -> Result<&'a Global, ValidationError> {
        self.globals
            .get(global_index as usize)
            .ok_or(ValidationError::from("Invalid global reference"))
    }

    pub fn get_local(&self, local_index: u32) -> Result<&ValueType, ValidationError> {
        self.locals
            .get(local_index as usize)
            .ok_or(ValidationError::from("Invalid local reference"))
    }
}
