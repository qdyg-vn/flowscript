mod casting;
mod io;
mod math;
mod compare;
mod introspection;

use crate::value::{ParentKind, Value};
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
    pub types: &'static [Signature],
}

#[derive(Debug)]
pub struct Signature {
    pub arguments: &'static [ParentKind],
    pub result: ParentKind,
    pub min_arity: u8,
    pub infinite_arity: bool,
}

pub const BUILTIN_TABLE: &[Builtin] = &[
    Builtin { name: "+", function: BuiltinFunction::Math(add),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Number, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::String, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "-", function: BuiltinFunction::Math(minus),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Number, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::String, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "*", function: BuiltinFunction::Math(multiply),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Number, min_arity: 1, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::String, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "print", function: BuiltinFunction::IO(print),
        types: &[
            Signature { arguments: &[ParentKind::Dynamic], result: ParentKind::Nil, min_arity: 0, infinite_arity: true },
        ]
    },
    Builtin { name: "to_string", function: BuiltinFunction::Casting(to_string),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::String], result: ParentKind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::String, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::Nil], result: ParentKind::String, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "to_integer", function: BuiltinFunction::Casting(to_integer),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Integer, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Integer, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Integer, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "to_float", function: BuiltinFunction::Casting(to_float),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Float, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Float, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Float, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "to_boolean", function: BuiltinFunction::Casting(to_boolean),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Boolean, min_arity: 1, infinite_arity: false },
            Signature { arguments: &[ParentKind::Nil], result: ParentKind::Boolean, min_arity: 1, infinite_arity: false },
        ]
    },
    Builtin { name: "==", function: BuiltinFunction::Compare(equal),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "<", function: BuiltinFunction::Compare(less),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: ">", function: BuiltinFunction::Compare(greater),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "!=", function: BuiltinFunction::Compare(not_equal),
        types: &[
            Signature { arguments: &[ParentKind::Number], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::String], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
            Signature { arguments: &[ParentKind::Boolean], result: ParentKind::Boolean, min_arity: 2, infinite_arity: true },
        ]
    },
    Builtin { name: "length", function: BuiltinFunction::Introspection(length),
        types: &[
            Signature { arguments: &[ParentKind::String], result: ParentKind::Integer, min_arity: 1, infinite_arity: false },
        ]
    },
];

pub fn get_builtin(index: u16) -> Builtin { BUILTIN_TABLE[index as usize] }

pub fn get_types(index: u16) -> &'static [Signature] { BUILTIN_TABLE[index as usize].types }
