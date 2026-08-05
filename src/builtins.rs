mod casting;
mod io;
mod math;
mod compare;
mod introspection;

use crate::value::{Kind, Value};
use casting::{to_string, to_integer, to_float, to_boolean};
use io::print;
use math::{add, minus, multiply};
use compare::{equal, less, greater, not_equal};
use introspection::length;
use crate::error_handler::Error;

#[derive(Copy, Clone, Debug)]
pub enum BuiltinFunction {
    Math(fn(&[Value]) -> Result<Value, Error>),
    IO(fn(&[Value])),
    Casting(fn(&[Value]) -> Result<Value, Error>),
    Compare(fn(&[Value]) -> Result<Value, Error>),
    Introspection(fn(&[Value]) -> Result<Value, Error>),
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
    Builtin { name: "<", function: BuiltinFunction::Compare(less), have_instruction: true,
        types: &[
            Signature { arguments: &[Kind::Integer], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::Float], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[Kind::String], result: Kind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: ">", function: BuiltinFunction::Compare(greater), have_instruction: true,
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
    Builtin { name: "length", function: BuiltinFunction::Introspection(length), have_instruction: false,
        types: &[
            Signature { arguments: &[Kind::String], result: Kind::Integer, min_arity: 1, infinite_arity: false },
        ]
    },
];

pub fn get_builtin(index: u16) -> Builtin { BUILTIN_TABLE[index as usize] }

pub fn get_types(index: u16) -> &'static [Signature] { BUILTIN_TABLE[index as usize].types }
