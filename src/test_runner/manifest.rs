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
    Action {
        line: u32,
        action: Action,
        expected: Vec<Value>,
    },

    #[serde(rename = "assert_exhaustion")]
    AssertExhaustion {
        line: u32,
        action: Action,
        text: String,
        expected: Vec<Value>,
    },

    #[serde(rename = "assert_invalid")]
    AssertInvalid {
        line: u32,
        filename: String,
        text: String,
    },

    #[serde(rename = "assert_malformed")]
    AssertMalformed(CommandAssertMalformed),

    #[serde(rename = "assert_trap")]
    AssertTrap {
        line: u32,
        action: Action,
        text: String,
        expected: Vec<Value>,
    },

    #[serde(rename = "assert_uninstantiable")]
    AssertUninstantiable {
        line: u32,
        filename: Option<String>,
        text: String,
    },

    #[serde(rename = "assert_unlinkable")]
    AssertUnlinkable {
        line: u32,
        filename: Option<String>,
        text: String,
    },

    #[serde(rename = "assert_return")]
    AssertReturn {
        line: u32,
        action: Action,
        expected: Vec<Value>,
    },

    #[serde(rename = "assert_return_arithmetic_nan")]
    AssertReturnArithmeticNan {
        line: u32,
        action: Action,
        expected: Vec<Value>,
    },

    #[serde(rename = "assert_return_canonical_nan")]
    AssertReturnCanonicalNan {
        line: u32,
        action: Action,
        expected: Vec<Value>,
    },

    #[serde(rename = "module")]
    Module(CommandModule),

    #[serde(rename = "register")]
    Register(CommandRegister),
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
    pub filename: String
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