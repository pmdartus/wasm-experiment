mod validation;
mod types;
mod instructions;
pub mod modules;

// use super::types::*;
// use types::*;

// const BASE: u64 = 2;

// #[derive(Debug)]
// pub struct ValidationError {
//     message: String,
// }

// impl ValidationError {
//     fn from(message: &str) -> ValidationError {
//         ValidationError {
//             message: String::from(message),
//         }
//     }

//     fn from_string(message: String) -> ValidationError {
//         ValidationError { message }
//     }
// }

// pub type ValidationResult = Result<(), ValidationError>;

// #[derive(Debug)]
// struct Context<'a> {
//     function_types: &'a Vec<FunctionType>,
//     functions: &'a Vec<Function>,
//     tables: &'a Vec<Table>,
//     memories: &'a Vec<Memory>,
//     globals: &'a Vec<Global>,
//     elements: &'a Vec<Element>,
//     locals: Vec<ValueType>,
// }

// #[derive(Debug, Clone, PartialEq)]
// enum Operand {
//     Value(ValueType),
//     Unknown,
// }

// #[derive(Debug)]
// struct ControlFrame {
//     label_types: Vec<ValueType>,
//     end_types: Vec<ValueType>,
//     height: usize,
//     unreachable: bool,
// }

// #[derive(Debug)]
// struct ExpressionContext {
//     operands: Vec<Operand>,
//     frames: Vec<ControlFrame>,
// }

// impl ExpressionContext {
//     fn new() -> ExpressionContext {
//         ExpressionContext {
//             operands: vec![],
//             frames: vec![],
//         }
//     }

//     fn top_frame(&self) -> Result<&ControlFrame, ValidationError> {
//         self.frames
//             .last()
//             .ok_or(ValidationError::from("Unexpected empty frame stack"))
//     }

//     fn push_operand(&mut self, operand: Operand) {
//         self.operands.push(operand);
//     }

//     fn pop_operand(&mut self) -> Result<Operand, ValidationError> {
//         if self.operands.len() == self.top_frame()?.height && self.top_frame()?.unreachable {
//             Ok(Operand::Unknown)
//         } else if self.operands.len() == self.top_frame()?.height {
//             Err(ValidationError::from("Invalid stack size"))
//         } else {
//             self.operands
//                 .pop()
//                 .ok_or(ValidationError::from("Unexpected empty operand stack"))
//         }
//     }

//     fn pop_operand_expected(&mut self, expected: &Operand) -> Result<Operand, ValidationError> {
//         let actual = self.pop_operand()?;

//         if actual == Operand::Unknown {
//             Ok(expected.clone())
//         } else if *expected == Operand::Unknown {
//             Ok(actual)
//         } else if actual != *expected {
//             Err(ValidationError::from_string(format!(
//                 "Mismatching type. Expected {:?} but received {:?}",
//                 expected, actual
//             )))
//         } else {
//             Ok(actual)
//         }
//     }

//     fn push_operands(&mut self, operands: Vec<Operand>) {
//         for operand in operands {
//             self.operands.push(operand);
//         }
//     }

//     fn pop_operands(&mut self, operands: &Vec<Operand>) -> ValidationResult {
//         let mut clone = operands.clone();
//         clone.reverse();

//         for operand in clone {
//             self.pop_operand_expected(&operand)?;
//         }

//         Ok(())
//     }

//     fn push_control(&mut self, label_types: Vec<ValueType>, end_types: Vec<ValueType>) {
//         self.frames.push(ControlFrame {
//             label_types,
//             end_types,
//             height: self.operands.len(),
//             unreachable: false,
//         });
//     }

//     // fn pop_control(&mut self) -> Result<Vec<ValueType>, ValidationError> {
//     //     let frame = self.top_frame()?;
//     //     let operand: Vec<Operand> = frame.end_types.iter().map(|t| Operand::Value(*t)).collect();
//     //     self.pop_operands(&operand)?;

//     //     if self.operands.len() != self.top_frame()?.height {
//     //         return Err(ValidationError::from("Mismatching frame height"))
//     //     }

//     //     // Should be safe to pop the top frame since it has been checked when accessing the top
//     //     // frame.
//     //     // self.frames.pop().unwrap();
//     //     Ok(frame.end_types.clone())
//     // }

//     fn unreachable(&mut self) -> ValidationResult {
//         // let frame = self.frames.last().unwrap();

//         // self.operands.resize(frame.height, Operand::Unknown);
//         // frame.unreachable = true;

//         Ok(())
//     }
// }

// fn get_function_type<'a>(
//     context: &'a Context,
//     function_type_index: u32,
// ) -> Result<&'a FunctionType, ValidationError> {
//     context
//         .function_types
//         .get(function_type_index as usize)
//         .ok_or(ValidationError::from("Invalid function type reference"))
// }

// fn get_function<'a>(
//     context: &'a Context,
//     function_index: u32,
// ) -> Result<&'a Function, ValidationError> {
//     context
//         .functions
//         .get(function_index as usize)
//         .ok_or(ValidationError::from("Invalid function reference"))
// }

// fn get_table<'a>(context: &'a Context, table_index: u32) -> Result<&'a Table, ValidationError> {
//     context
//         .tables
//         .get(table_index as usize)
//         .ok_or(ValidationError::from("Invalid table reference"))
// }

// fn get_memory<'a>(context: &'a Context, memory_index: u32) -> Result<&'a Memory, ValidationError> {
//     context
//         .memories
//         .get(memory_index as usize)
//         .ok_or(ValidationError::from("Invalid memory reference"))
// }

// fn get_global<'a>(context: &'a Context, global_index: u32) -> Result<&'a Global, ValidationError> {
//     context
//         .globals
//         .get(global_index as usize)
//         .ok_or(ValidationError::from("Invalid global reference"))
// }

// fn get_local<'a>(context: &'a Context, local_index: u32) -> Result<&'a ValueType, ValidationError> {
//     context
//         .locals
//         .get(local_index as usize)
//         .ok_or(ValidationError::from("Invalid local reference"))
// }