use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub source_filename: String,
    pub commands: Vec<Command>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "action")]
    Action(CommandAction),

    #[serde(rename = "assert_exhaustion")]
    AssertExhaustion(CommandAssertExhaustion),

    #[serde(rename = "assert_invalid")]
    AssertInvalid(CommandAssertInvalid),

    #[serde(rename = "assert_malformed")]
    AssertMalformed(CommandAssertMalformed),

    #[serde(rename = "assert_trap")]
    AssertTrap(CommandAssertTrap),

    #[serde(rename = "assert_uninstantiable")]
    AssertUninstantiable(CommandAssertUninstantiable),

    #[serde(rename = "assert_unlinkable")]
    AssertUnlinkable(CommandAssertUnlinkable),

    #[serde(rename = "assert_return")]
    AssertReturn(CommandAssertReturn),

    #[serde(rename = "assert_return_arithmetic_nan")]
    AssertReturnArithmeticNan(CommandAssertReturnArithmeticNan),

    #[serde(rename = "assert_return_canonical_nan")]
    AssertReturnCanonicalNan(CommandAssertReturnCanonicalNan),

    #[serde(rename = "module")]
    Module(CommandModule),

    #[serde(rename = "register")]
    Register(CommandRegister),
}

#[derive(Deserialize, Debug)]
pub struct CommandAction {
    pub line: u32,
    pub action: Action,
    pub expected: Vec<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertExhaustion {
    pub line: u32,
    pub action: Action,
    pub text: String,
    pub expected: Vec<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertInvalid {
    pub line: u32,
    pub filename: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertTrap {
    pub line: u32,
    pub action: Action,
    pub text: String,
    pub expected: Vec<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertUninstantiable {
    pub line: u32,
    pub filename: Option<String>,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertUnlinkable {
    pub line: u32,
    pub filename: Option<String>,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertReturn {
    pub line: u32,
    pub action: Action,
    pub expected: Vec<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertReturnArithmeticNan {
    pub line: u32,
    pub action: Action,
    pub expected: Vec<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertReturnCanonicalNan {
    pub line: u32,
    pub action: Action,
    pub expected: Vec<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CommandAssertMalformed {
    pub line: u32,
    pub filename: String,
    pub text: String,
}

#[derive(Deserialize, Debug)]
pub struct CommandModule {
    pub line: u32,
    pub filename: String,
}

#[derive(Deserialize, Debug)]
pub struct CommandRegister {
    pub line: u32,
    pub name: Option<String>,
    #[serde(alias = "as")]
    pub alias: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "invoke")]
    Invoke { field: String, args: Vec<Value> },

    #[serde(rename = "get")]
    Get {
        field: String,
        module: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Value {
    #[serde(rename = "i32")]
    I32 { value: Option<String> },
    #[serde(rename = "i64")]
    I64 { value: Option<String> },
    #[serde(rename = "f32")]
    F32 { value: Option<String> },
    #[serde(rename = "f64")]
    F64 { value: Option<String> },
}
