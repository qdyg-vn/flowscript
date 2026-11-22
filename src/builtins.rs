mod casting;
mod io;
mod math;
mod compare;
mod collection;

use crate::value::{Kind, Value};
use casting::{to_string, to_integer, to_float, to_boolean};
use io::print;
use math::{add, minus, multiply};
use compare::{equal, lower_than, greater_than, lower_than_or_equal, greater_than_or_equal, not_equal};
use collection::{length, push, pop};
use crate::error_handler::Error;

#[derive(Copy, Clone, Debug)]
pub enum BuiltinFunction {
    Math(fn(Vec<Value>) -> Result<Value, Error>),
    IO(fn(Vec<Value>)),
    Casting(fn(Vec<Value>) -> Result<Value, Error>),
    Compare(fn(Vec<Value>) -> Result<Value, Error>),
    Collection(fn(Vec<Value>) -> Result<Value, Error>),
}

#[derive(Copy, Clone, Debug)]
pub struct Builtin {
    pub name: &'static str,
    pub function: BuiltinFunction,
    pub have_instruction: bool,
    pub types: &'static [Signature],
}

#[derive(Debug)]
pub struct Signature {
    pub fixed: &'static [Kind],
    pub variadic: &'static [Kind],
    pub result: Kind,
}

pub const BUILTIN_TABLE: &[Builtin] = &[
    Builtin { name: "+", function: BuiltinFunction::Math(add), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer], variadic: &[Kind::Integer], result: Kind::Integer },
            Signature { fixed: &[Kind::Float], variadic: &[Kind::Float], result: Kind::Float },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::String },
        ]
    },
    Builtin { name: "-", function: BuiltinFunction::Math(minus), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer], variadic: &[Kind::Integer], result: Kind::Integer },
            Signature { fixed: &[Kind::Float], variadic: &[Kind::Float], result: Kind::Float },
        ]
    },
    Builtin { name: "*", function: BuiltinFunction::Math(multiply), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Integer },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Float },
        ]
    },
    Builtin { name: "print", function: BuiltinFunction::IO(print), have_instruction: false,
        types: &[
            Signature { fixed: &[], variadic: &[], result: Kind::Nil },
            Signature { fixed: &[Kind::Integer], variadic: &[], result: Kind::Nil },
            Signature { fixed: &[Kind::Float], variadic: &[], result: Kind::Nil },
            Signature { fixed: &[Kind::String], variadic: &[], result: Kind::Nil },
            Signature { fixed: &[Kind::Boolean], variadic: &[], result: Kind::Nil },
            Signature { fixed: &[Kind::Nil], variadic: &[], result: Kind::Nil },
            Signature { fixed: &[Kind::Array], variadic: &[], result: Kind::Nil },
        ]
    },
    Builtin { name: "to_string", function: BuiltinFunction::Casting(to_string), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::Integer], variadic: &[], result: Kind::String },
            Signature { fixed: &[Kind::Float], variadic: &[], result: Kind::String },
            Signature { fixed: &[Kind::String], variadic: &[], result: Kind::String },
            Signature { fixed: &[Kind::Boolean], variadic: &[], result: Kind::String },
            Signature { fixed: &[Kind::Nil], variadic: &[], result: Kind::String },
        ]
    },
    Builtin { name: "to_integer", function: BuiltinFunction::Casting(to_integer), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::Integer], variadic: &[], result: Kind::Integer },
            Signature { fixed: &[Kind::Float], variadic: &[], result: Kind::Integer },
            Signature { fixed: &[Kind::String], variadic: &[], result: Kind::Integer },
            Signature { fixed: &[Kind::Boolean], variadic: &[], result: Kind::Integer },
        ]
    },
    Builtin { name: "to_float", function: BuiltinFunction::Casting(to_float), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::Integer], variadic: &[], result: Kind::Float },
            Signature { fixed: &[Kind::Float], variadic: &[], result: Kind::Float },
            Signature { fixed: &[Kind::String], variadic: &[], result: Kind::Float },
            Signature { fixed: &[Kind::Boolean], variadic: &[], result: Kind::Float },
        ]
    },
    Builtin { name: "to_boolean", function: BuiltinFunction::Casting(to_boolean), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::Integer], variadic: &[], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float], variadic: &[], result: Kind::Boolean },
            Signature { fixed: &[Kind::String], variadic: &[], result: Kind::Boolean },
            Signature { fixed: &[Kind::Boolean], variadic: &[], result: Kind::Boolean },
            Signature { fixed: &[Kind::Nil], variadic: &[], result: Kind::Boolean },
        ]
    },
    Builtin { name: "==", function: BuiltinFunction::Compare(equal), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Boolean },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::Boolean },
            Signature { fixed: &[Kind::Boolean, Kind::Boolean], variadic: &[Kind::Boolean], result: Kind::Boolean },
        ]
    },
    Builtin { name: "<", function: BuiltinFunction::Compare(lower_than), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Boolean },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::Boolean },
        ]
    },
    Builtin { name: ">", function: BuiltinFunction::Compare(greater_than), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Boolean },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::Boolean },
        ]
    },
    Builtin { name: "<=", function: BuiltinFunction::Compare(lower_than_or_equal), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Boolean },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::Boolean },
        ]
    },
    Builtin { name: ">=", function: BuiltinFunction::Compare(greater_than_or_equal), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Boolean },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::Boolean },
        ]
    },
    Builtin { name: "!=", function: BuiltinFunction::Compare(not_equal), have_instruction: true,
        types: &[
            Signature { fixed: &[Kind::Integer, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Boolean },
            Signature { fixed: &[Kind::Float, Kind::Float], variadic: &[Kind::Float], result: Kind::Boolean },
            Signature { fixed: &[Kind::String, Kind::String], variadic: &[Kind::String], result: Kind::Boolean },
            Signature { fixed: &[Kind::Boolean, Kind::Boolean], variadic: &[Kind::Boolean], result: Kind::Boolean },
        ]
    },
    Builtin { name: "length", function: BuiltinFunction::Collection(length), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::String], variadic: &[], result: Kind::Integer },
            Signature { fixed: &[Kind::Array], variadic: &[], result: Kind::Integer },
        ]
    },
    Builtin { name: "push", function: BuiltinFunction::Collection(push), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::Array, Kind::Integer], variadic: &[Kind::Integer], result: Kind::Array },
            Signature { fixed: &[Kind::Array, Kind::Float], variadic: &[Kind::Float], result: Kind::Array },
            Signature { fixed: &[Kind::Array, Kind::String], variadic: &[Kind::String], result: Kind::Array },
            Signature { fixed: &[Kind::Array, Kind::Boolean], variadic: &[Kind::Boolean], result: Kind::Array },
            Signature { fixed: &[Kind::Array, Kind::Nil], variadic: &[Kind::Nil], result: Kind::Array },
            Signature { fixed: &[Kind::Array, Kind::Array], variadic: &[Kind::Array], result: Kind::Array },
        ]
    },
    Builtin { name: "pop", function: BuiltinFunction::Collection(pop), have_instruction: false,
        types: &[
            Signature { fixed: &[Kind::Array], variadic: &[], result: Kind::Array },
        ]
    },
];

pub fn get_builtin(index: u16) -> Builtin { BUILTIN_TABLE[index as usize] }

pub fn get_signatures(index: u16) -> &'static [Signature] { BUILTIN_TABLE[index as usize].types }
