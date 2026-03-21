mod math;
mod io;

use crate::value::Value;

#[derive(Copy, Clone, Debug)]
pub enum BuiltinFunction {
    Math(fn(&[Value]) -> Result<Value, String>),
    IO(fn(&[Value])),
}

#[derive(Copy, Clone, Debug)]
pub struct Builtin {
    pub name: &'static str,
    pub function: BuiltinFunction
}

pub const BUILTIN_TABLE: &[Builtin] = &[
];

pub fn get_builtin(index: u16) -> Builtin {
    BUILTIN_TABLE[index as usize]
}
