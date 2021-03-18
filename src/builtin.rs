use std::collections::HashMap;

use crate::func::Closure;
use crate::value::Value;
use crate::State;

macro_rules! add_func {
    ($m:ident, $name:ident) => {
        $m.insert(
            Value::String(stringify!($name).to_string()),
            Value::Function(Closure::with_builtin($name, 0)),
        );
    };
}

pub fn add_builtin_func(m: &mut HashMap<Value, Value>) {
    add_func!(m, print);
}

fn print(state: &mut State) -> usize {
    println!(
        "{}",
        (1..=state.top())
            .map(|index| {
                state.get_rk((index - 1) as i32);
                format!("{}", state.pop_value())
            })
            .collect::<Vec<String>>()
            .join(" ")
    );
    0
}
