mod casting;
mod io;
mod math;
mod compare;
mod introspection;

use crate::value::Value;
use casting::to_string;
use io::print;
use math::{add, minus};
use compare::{equal, less, greater, not_equal};
use introspection::length;
use crate::error_handler::Error;

#[derive(Copy, Clone, Debug)]
pub enum BuiltinFunction {
    Math(fn(&[Value]) -> Result<Value, String>),
    IO(fn(&[Value])),
    Casting(fn(&[Value]) -> Value),
    Compare(fn(&[Value]) -> Value),
    Introspection(fn(&[Value]) -> Result<Value, Error>),
}

#[derive(Copy, Clone, Debug)]
pub struct Builtin {
    pub name: &'static str,
    pub function: BuiltinFunction,
}

pub const BUILTIN_TABLE: &[Builtin] = &[
    Builtin { name: "+", function: BuiltinFunction::Math(add) },
    Builtin { name: "-", function: BuiltinFunction::Math(minus) },
    Builtin { name: "print", function: BuiltinFunction::IO(print) },
    Builtin { name: "string", function: BuiltinFunction::Casting(to_string) },
    Builtin { name: "==", function: BuiltinFunction::Compare(equal) },
    Builtin { name: "<", function: BuiltinFunction::Compare(less) },
    Builtin { name: ">", function: BuiltinFunction::Compare(greater) },
    Builtin { name: "!=", function: BuiltinFunction::Compare(not_equal) },
    Builtin { name: "length", function: BuiltinFunction::Introspection(length) },
];

pub fn get_builtin(index: u16) -> Builtin {
    BUILTIN_TABLE[index as usize]
}
