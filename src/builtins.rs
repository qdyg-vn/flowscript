mod casting;
mod io;
mod math;
mod compare;

use crate::value::Value;
use casting::to_string;
use io::print;
use math::add;
use compare::equal;

#[derive(Copy, Clone, Debug)]
pub enum BuiltinFunction {
    Math(fn(&[Value]) -> Result<Value, String>),
    IO(fn(&[Value])),
    Casting(fn(&[Value]) -> Value),
    Compare(fn(&[Value]) -> Value),
}

#[derive(Copy, Clone, Debug)]
pub struct Builtin {
    pub name: &'static str,
    pub function: BuiltinFunction,
}

pub const BUILTIN_TABLE: &[Builtin] = &[
    Builtin { name: "+", function: BuiltinFunction::Math(add) },
    Builtin { name: "print", function: BuiltinFunction::IO(print) },
    Builtin { name: "string", function: BuiltinFunction::Casting(to_string) },
    Builtin { name: "==", function: BuiltinFunction::Compare(equal) }
];

pub fn get_builtin(index: u16) -> Builtin {
    BUILTIN_TABLE[index as usize]
}
