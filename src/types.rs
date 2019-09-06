// TODO: Understand why Copy and Clone are always applied at the same time.
// More details: https://doc.rust-lang.org/std/marker/trait.Copy.html
/// https://webassembly.github.io/spec/core/syntax/types.html#value-types
#[derive(Debug, Copy, Clone)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#function-types
pub type FunctionType = (Vec<ValueType>, Vec<ValueType>);

/// https://webassembly.github.io/spec/core/syntax/types.html#limits
#[derive(Debug)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#memory-types
#[derive(Debug)]
pub struct MemoryType {
    pub limits: Limits,
}

#[derive(Debug)]
pub enum ElementType {
    FuncRef,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#table-types
#[derive(Debug)]
pub struct TableType {
    pub limits: Limits,
    pub element_type: ElementType,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#global-types
#[derive(Debug)]
pub enum GlobalTypeMutability {
    Const,
    Var,
}
#[derive(Debug)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutability: GlobalTypeMutability,
}

#[derive(Debug, Copy, Clone)]
pub enum BlockType {
    Void,
    Return(ValueType),
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryArg {
    pub align: u32,
    pub offset: u32,
}

/// https://webassembly.github.io/spec/core/syntax/instructions.html#expressions
pub type Expression = Vec<Instruction>;

/// https://webassembly.github.io/spec/core/syntax/instructions.html
#[derive(Debug, Clone)]
pub enum Instruction {
    // Control flow instructions
    Unreachable,
    Nop,
    Block(BlockType, Vec<Instruction>),
    Loop(BlockType, Vec<Instruction>),
    If(BlockType, Vec<Instruction>, Option<Vec<Instruction>>),
    Br(Index),
    BrIf(Index),
    BrTable(Vec<Index>, Index),
    Return,
    Call(Index),
    CallIndirect(Index),

    // Parametric instructions
    Drop,
    Select,

    // Variable instructions
    LocalGet(Index),
    LocalSet(Index),
    LocalTee(Index),
    GlobalGet(Index),
    GlobalSet(Index),

    // Memory instructions
    I32Load(MemoryArg),
    I64Load(MemoryArg),
    F32Load(MemoryArg),
    F64Load(MemoryArg),
    I32Load8S(MemoryArg),
    I32Load8U(MemoryArg),
    I32Load16S(MemoryArg),
    I32Load16U(MemoryArg),
    I64Load8S(MemoryArg),
    I64Load8U(MemoryArg),
    I64Load16S(MemoryArg),
    I64Load16U(MemoryArg),
    I64Load32S(MemoryArg),
    I64Load32U(MemoryArg),
    I32Store(MemoryArg),
    I64Store(MemoryArg),
    F32Store(MemoryArg),
    F64Store(MemoryArg),
    I32Store8(MemoryArg),
    I32Store16(MemoryArg),
    I64Store8(MemoryArg),
    I64Store16(MemoryArg),
    I64Store32(MemoryArg),
    MemorySize,
    MemoryGrow,

    // Constants instructions
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),

    // Comparison operators
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    I32LtU,
    I32GtS,
    I32GtU,
    I32LeS,
    I32LeU,
    I32GeS,
    I32GeU,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtS,
    I64LtU,
    I64GtS,
    I64GtU,
    I64LeS,
    I64LeU,
    I64GeS,
    I64GeU,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,

    // Numeric operators
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I64Clz,
    I64Ctz,
    I64Popcnt,
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64DivU,
    I64RemS,
    I64RemU,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Rotl,
    I64Rotr,
    F32Abs,
    F32Neg,
    F32Ceil,
    F32Floor,
    F32Trunc,
    F32Nearest,
    F32Sqrt,
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Min,
    F32Max,
    F32CopySign,
    F64Abs,
    F64Neg,
    F64Ceil,
    F64Floor,
    F64Trunc,
    F64Nearest,
    F64Sqrt,
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Min,
    F64Max,
    F64CopySign,

    // Conversions
    I32WrapI64,
    I32TruncSF32,
    I32TruncUF32,
    I32TruncSF64,
    I32TruncUF64,
    I64ExtendSI32,
    I64ExtendUI32,
    I64TruncSF32,
    I64TruncUF32,
    I64TruncSF64,
    I64TruncUF64,
    F32ConvertSI32,
    F32ConvertUI32,
    F32ConvertSI64,
    F32ConvertUI64,
    F32DemoteF64,
    F64ConvertSI32,
    F64ConvertUI32,
    F64ConvertSI64,
    F64ConvertUI64,
    F64PromoteF32,

    // Reinterpretations
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#indices
#[derive(Debug, Copy, Clone)]
pub enum Index {
    Type(u32),
    Function(u32),
    Table(u32),
    Memory(u32),
    Global(u32),
    Local(u32),
    Label(u32),
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#functions
#[derive(Debug)]
pub struct Function {
    pub function_type: Index,
    pub locals: Vec<(u32, ValueType)>,
    pub body: Expression,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#tables
#[derive(Debug)]
pub struct Table {
    pub table_type: TableType,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#memories
#[derive(Debug)]
pub struct Memory {
    pub memory_type: MemoryType,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#globals
#[derive(Debug)]
pub struct Global {
    pub global_type: GlobalType,
    pub init: Expression,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#element-segments
#[derive(Debug)]
pub struct Element {
    pub table: Index,
    pub offset: Expression,
    pub init: Vec<Index>,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#data-segments
#[derive(Debug)]
pub struct Data {
    pub data: Index,
    pub offset: Expression,
    pub init: Vec<u8>,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#start-function
#[derive(Debug)]
pub struct StartFunction {
    pub function: Index,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#exports
#[derive(Debug)]
pub struct Export {
    pub name: String,
    pub descriptor: Index,
}

/// https://webassembly.github.io/spec/core/syntax/modules.html#imports
#[derive(Debug)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub descriptor: ImportDescriptor,
}
#[derive(Debug)]
pub enum ImportDescriptor {
    Function(u32),
    Table(TableType),
    Memory(MemoryType),
    Global(GlobalType)
}

pub type CustomSection<'a> = (String, &'a [u8]);

/// https://webassembly.github.io/spec/core/syntax/modules.html#modules
#[derive(Debug)]
pub struct Module<'a> {
    pub custom_sections: Vec<CustomSection<'a>>,
    pub function_types: Vec<FunctionType>,
    pub functions: Vec<Function>,
    pub tables: Vec<Table>,
    pub memories: Vec<Memory>,
    pub globals: Vec<Global>,
    pub elements: Vec<Element>,
    pub data: Vec<Data>,
    pub start: Option<StartFunction>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
}