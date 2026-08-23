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
    pub arguments: &'static [Kind],
    pub result: Kind,
    pub min_arity: u8,
    pub infinite_arity: bool,
}

pub const BUILTIN_TABLE: &[Builtin] = &[
    Builtin { name: "+", function: BuiltinFunction::Math(add), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Integer, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Float, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::String, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "-", function: BuiltinFunction::Math(minus), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Integer, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Float, min_arity: 1, infinite_arity: true },
        ]
    },
    Builtin { name: "*", function: BuiltinFunction::Math(multiply), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Integer, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Float, min_arity: 1, infinite_arity: true },
        ]
    },
    Builtin { name: "print", function: BuiltinFunction::IO(print), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Nil, min_arity: 0, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Nil, min_arity: 0, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Nil, min_arity: 0, infinite_arity: true },
            Signature { arguments: &[Kind::Boolean], result: Kind::Nil, min_arity: 0, infinite_arity: true },
            Signature { arguments: &[Kind::Nil], result: Kind::Nil, min_arity: 0, infinite_arity: true },
            Signature { arguments: &[Kind::Array], result: Kind::Nil, min_arity: 0, infinite_arity: true },
        ]
    },
    Builtin { name: "to_string", function: BuiltinFunction::Casting(to_string), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Float], result: Kind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::String], result: Kind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Boolean], result: Kind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Nil], result: Kind::String, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "to_integer", function: BuiltinFunction::Casting(to_integer), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Integer, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Float], result: Kind::Integer, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::String], result: Kind::Integer, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Boolean], result: Kind::Integer, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "to_float", function: BuiltinFunction::Casting(to_float), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Float, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Float], result: Kind::Float, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::String], result: Kind::Float, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Boolean], result: Kind::Float, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "to_boolean", function: BuiltinFunction::Casting(to_boolean), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Boolean], result: Kind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[Kind::Nil], result: Kind::Boolean, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "==", function: BuiltinFunction::Compare(equal), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Boolean], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "<", function: BuiltinFunction::Compare(lower_than), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: ">", function: BuiltinFunction::Compare(greater_than), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "<=", function: BuiltinFunction::Compare(lower_than_or_equal), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: ">=", function: BuiltinFunction::Compare(greater_than_or_equal), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "!=", function: BuiltinFunction::Compare(not_equal), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Boolean], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "length", function: BuiltinFunction::Collection(length), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::String], result: Kind::Integer, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "push", function: BuiltinFunction::Collection(push), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Integer, Kind::Array], result: Kind::Array, min_arity: 2, infinite_arity: false },
            Signature { arguments: &[Kind::Float, Kind::Array], result: Kind::Array, min_arity: 2, infinite_arity: false },
            Signature { arguments: &[Kind::String, Kind::Array], result: Kind::Array, min_arity: 2, infinite_arity: false },
            Signature { arguments: &[Kind::Boolean, Kind::Array], result: Kind::Array, min_arity: 2, infinite_arity: false },
            Signature { arguments: &[Kind::Nil, Kind::Array], result: Kind::Array, min_arity: 2, infinite_arity: false },
            Signature { arguments: &[Kind::Array, Kind::Array], result: Kind::Array, min_arity: 2, infinite_arity: false },
        ]
    },
    Builtin { name: "pop", function: BuiltinFunction::Collection(pop), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::Array], result: Kind::Array, min_arity: 1, infinite_arity: false },
        ]
    },
];

pub fn get_builtin(index: u16) -> Builtin { BUILTIN_TABLE[index as usize] }

pub fn get_types(index: u16) -> &'static [Signature] { BUILTIN_TABLE[index as usize].types }
