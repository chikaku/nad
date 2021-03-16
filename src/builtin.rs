use crate::state::State;
use crate::value::Value;
use crate::Func;
use std::collections::HashMap;

macro_rules! add_func {
    ($m:ident, $name:ident) => {
        $m.insert(
            Value::String(stringify!($name).to_string()),
            Value::Function(Func::Builtin($name)),
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
            .collect::<String>()
    );
    0
}
