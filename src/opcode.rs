use crate::instruction::Instruction;
use crate::state::State;
use crate::value::Value;

#[derive(Eq, PartialEq)]
pub enum Mode {
    IABC,  // [  B:9  ][  C:9  ][ A:8  ][OP:6]
    IABx,  // [      Bx:18     ][ A:8  ][OP:6]
    IAsBx, // [     sBx:18     ][ A:8  ][OP:6]
    IAx,   // [           Ax:26        ][OP:6]
}

#[derive(Eq, PartialEq)]
pub enum ArgType {
    N, // not used
    U, // used
    R, // register or jump offset
    K, // index of register or constant
}

pub struct Code {
    test_flag: u8,
    seta_flag: u8,
    pub argb_mode: ArgType,
    pub argc_mode: ArgType,
    pub op_mode: Mode,
    pub name: &'static str,
    exec: fn(Instruction, &mut State),
}

macro_rules! math1 {
    ($op:tt) => {
        |ins: Instruction, state: &mut State| {
            let (a, b, _) = ins.abc();
            state.push_index(b + 1);
            let val = state.pop_value();
            match $op val {
                Ok(res) => {
                    state.push_value(res);
                    state.replace(a + 1);
                },
                Err(e) => panic!("{}", e),
            }
        }
    };
}

macro_rules! math2 {
    ($op:tt) => {
        |ins: Instruction, state: &mut State| {
            let (a, b, c) = ins.abc();
            state.get_rk(b);
            state.get_rk(c);
            let vb = state.pop_value();
            let va = state.pop_value();
            match va $op vb {
                Ok(res) => {
                    state.push_value(res);
                    state.replace(a + 1);
                },
                Err(e) => panic!("{}", e),
            }
        }
    };
}

macro_rules! cmp {
    ($op:tt) => {
        |ins: Instruction, state: &mut State| {
            let (a, b, c) = ins.abc();
            state.get_rk(b);
            state.get_rk(c);
            if state.compare(-2, -1, "$op") != (a != 0) {
                state.add_pc(1);
            }
            state.pop(2);
        }
    };
}

macro_rules! code {
    ($test:expr, $seta:expr, $argb:ident, $argc:ident, $mode:ident, $name:expr, $exec:expr) => {
        Code {
            test_flag: $test,
            seta_flag: $seta,
            argb_mode: ArgType::$argb,
            argc_mode: ArgType::$argc,

            op_mode: Mode::$mode,
            name: $name,
            exec: $exec,
        }
    };
}

pub const ALL: &'static [Code] = &[
    /*    T  A  B  C  mode         name    */
    code!(0, 1, R, N, IABC /* */, "MOVE    ", move_), // R(A) := R(B)
    code!(0, 1, K, N, IABx /* */, "LOADK   ", load_const), // R(A) := Kst(Bx)
    code!(0, 1, N, N, IABx /* */, "LOADKX  ", load_constx), // R(A) := Kst(extra arg)
    code!(0, 1, U, U, IABC /* */, "LOADBOOL", load_bool), // R(A) := (bool)B; if (C) pc++
    code!(0, 1, U, N, IABC /* */, "LOADNIL ", load_nil), // R(A), R(A+1), ..., R(A+B) := nil
    code!(0, 1, U, N, IABC /* */, "GETUPVAL", unimplement), // R(A) := UpValue[B]
    code!(0, 1, U, K, IABC /* */, "GETTABUP", unimplement), // R(A) := UpValue[B][RK(C)]
    code!(0, 1, R, K, IABC /* */, "GETTABLE", unimplement), // R(A) := R(B)[RK(C)]
    code!(0, 0, K, K, IABC /* */, "SETTABUP", unimplement), // UpValue[A][RK(B)] := RK(C)
    code!(0, 0, U, N, IABC /* */, "SETUPVAL", unimplement), // UpValue[B] := R(A)
    code!(0, 0, K, K, IABC /* */, "SETTABLE", unimplement), // R(A)[RK(B)] := RK(C)
    code!(0, 1, U, U, IABC /* */, "NEWTABLE", unimplement), // R(A) := {} (size = B,C)
    code!(0, 1, R, K, IABC /* */, "SELF    ", unimplement), // R(A+1) := R(B); R(A) := R(B)[RK(C)]
    code!(0, 1, K, K, IABC /* */, "ADD     ", math2!(+)), // R(A) := RK(B) + RK(C)
    code!(0, 1, K, K, IABC /* */, "SUB     ", math2!(-)), // R(A) := RK(B) - RK(C)
    code!(0, 1, K, K, IABC /* */, "MUL     ", math2!(*)), // R(A) := RK(B) * RK(C)
    code!(0, 1, K, K, IABC /* */, "MOD     ", math2!(%)), // R(A) := RK(B) % RK(C)
    code!(0, 1, K, K, IABC /* */, "POW     ", math2!(*)), // R(A) := RK(B) ^ RK(C) TODO: implement pow
    code!(0, 1, K, K, IABC /* */, "DIV     ", math2!(/)), // R(A) := RK(B) / RK(C)
    code!(0, 1, K, K, IABC /* */, "IDIV    ", math2!(/)), // R(A) := RK(B) // RK(C)
    code!(0, 1, K, K, IABC /* */, "BAND    ", math2!(&)), // R(A) := RK(B) & RK(C)
    code!(0, 1, K, K, IABC /* */, "BOR     ", math2!(|)), // R(A) := RK(B) | RK(C)
    code!(0, 1, K, K, IABC /* */, "BXOR    ", math2!(^)), // R(A) := RK(B) ~ RK(C)
    code!(0, 1, K, K, IABC /* */, "SHL     ", math2!(<<)), // R(A) := RK(B) << RK(C)
    code!(0, 1, K, K, IABC /* */, "SHR     ", math2!(>>)), // R(A) := RK(B) >> RK(C)
    code!(0, 1, R, N, IABC /* */, "UNM     ", math1!(-)), // R(A) := -R(B)
    code!(0, 1, R, N, IABC /* */, "BNOT    ", math1!(!)), // R(A) := ~R(B)
    code!(0, 1, R, N, IABC /* */, "NOT     ", not),       // R(A) := not R(B)
    code!(0, 1, R, N, IABC /* */, "LEN     ", len),       // R(A) := length of R(B)
    code!(0, 1, R, R, IABC /* */, "CONCAT  ", concat),    // R(A) := R(B).. ... ..R(C)
    code!(0, 0, R, N, IAsBx /**/, "JMP     ", jmp /*   */), // pc+=sBx; if (A) close all upvalues >= R(A - 1)
    code!(1, 0, K, K, IABC /* */, "EQ      ", cmp!(==)),    // if ((RK(B) == RK(C)) ~= A) then pc++
    code!(1, 0, K, K, IABC /* */, "LT      ", cmp!(<)),     // if ((RK(B) <  RK(C)) ~= A) then pc++
    code!(1, 0, K, K, IABC /* */, "LE      ", cmp!(<=)),    // if ((RK(B) <= RK(C)) ~= A) then pc++
    code!(1, 0, N, U, IABC /* */, "TEST    ", test),        // if not (R(A) <=> C) then pc++
    code!(1, 1, R, U, IABC /* */, "TESTSET ", test_set), // if (R(B) <=> C) then R(A) := R(B) else pc++
    code!(0, 1, U, U, IABC /* */, "CALL    ", unimplement), // R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
    code!(0, 1, U, U, IABC /* */, "TAILCALL", unimplement), // return R(A)(R(A+1), ... ,R(A+B-1))
    code!(0, 0, U, N, IABC /* */, "RETURN  ", unimplement), // return R(A), ... ,R(A+B-2)
    code!(0, 1, R, N, IAsBx /**/, "FORLOOP ", unimplement), // R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    code!(0, 1, R, N, IAsBx /**/, "FORPREP ", unimplement), // R(A)-=R(A+2); pc+=sBx
    code!(0, 0, N, U, IABC /* */, "TFORCALL", unimplement), // R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
    code!(0, 1, R, N, IAsBx /**/, "TFORLOOP", unimplement), // if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
    code!(0, 0, U, U, IABC /* */, "SETLIST ", unimplement), // R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    code!(0, 1, U, N, IABx /* */, "CLOSURE ", unimplement), // R(A) := closure(KPROTO[Bx])
    code!(0, 1, U, N, IABC /* */, "VARARG  ", unimplement), // R(A), R(A+1), ..., R(A+B-2) = vararg
    code!(0, 0, U, U, IAx /*  */, "EXTRAARG", unimplement), // extra (larger) argument for previous opcode
];

fn unimplement(_: Instruction, _: &mut State) {
    unimplemented!();
}

fn move_(ins: Instruction, state: &mut State) {
    let (a, b, _) = ins.abc();
    state.copy(b + 1, a + 1);
}

fn jmp(ins: Instruction, state: &mut State) {
    let (a, sbx) = ins.asbx();
    state.add_pc(sbx);
    assert_eq!(a, 0, "unimplemented")
}

fn load_nil(ins: Instruction, state: &mut State) {
    let (a, b, _) = ins.abc();
    state.push_value(Value::Nil);
    ((a + 1)..(a + b)).for_each(|index| state.copy(-1, index));
    state.pop(1);
}

fn load_bool(ins: Instruction, state: &mut State) {
    let (a, b, c) = ins.abc();
    state.push_value(Value::Bool(b != 0));
    state.replace(a);
    if c != 0 {
        state.add_pc(1)
    }
}

/// load constant index from current instruction
fn load_const(ins: Instruction, state: &mut State) {
    let (a, bx) = ins.abx();
    assert!(bx >= 0);
    state.get_const(bx as usize);
    state.replace(a + 1);
}

/// load constant index from next instruction(`EXTRAARG`)
fn load_constx(ins: Instruction, state: &mut State) {
    let (a, _) = ins.abx();
    let ax = state.fetch().ax();
    assert!(ax >= 0);
    state.get_const(ax as usize);
    state.replace(a + 1);
}

fn len(ins: Instruction, state: &mut State) {
    let (a, b, _) = ins.abc();
    state.len(b + 1);
    state.replace(a + 1);
}

fn concat(ins: Instruction, state: &mut State) {
    let (a, b, c) = ins.abc();
    let (a, b, c) = (a + 1, b + 1, c + 1);

    assert!(c > b);
    let size = (c - b + 1) as usize;
    state.check_stack(size);
    (b..=c).for_each(|i| state.push_index(i));
    state.concat(size);
    state.replace(a + 1);
}

fn not(ins: Instruction, state: &mut State) {
    let (a, b, _) = ins.abc();
    state.push_value(Value::Bool(!state.to_boolean(b + 1)));
    state.replace(a + 1);
}

fn test_set(ins: Instruction, state: &mut State) {
    let (a, b, c) = ins.abc();
    if state.to_boolean(b + 1) == (c != 0) {
        state.copy(b + 1, a + 1);
    } else {
        state.add_pc(1);
    }
}

fn test(ins: Instruction, state: &mut State) {
    let (a, _, c) = ins.abc();
    if state.to_boolean(a + 1) != (c != 0) {
        state.add_pc(1);
    }
}

fn for_prep(ins: Instruction, state: &mut State) {
    let (a, sbx) = ins.asbx();
    let a = a + 1;

    state.push_index(2);
    state.push_index(a + 2);

    let vb = state.pop_value();
    let va = state.pop_value();
    state.push_value((va - vb).unwrap());
    state.replace(a);

    state.add_pc(sbx);
}

fn for_loop(ins: Instruction, state: &mut State) {
    let (a, sbx) = ins.asbx();
    let a = a + 1;

    state.push_index(a + 2);
    state.push_index(2);

    let vb = state.pop_value();
    let va = state.pop_value();
    state.push_value((va + vb).unwrap());
    state.replace(a);

    let postive_step = state.to_number(a + 2) > 0.0;
    if (postive_step && state.compare(a, a + 1, ">"))
        || (!postive_step && state.compare(a + 1, a, ">"))
    {
        state.add_pc(sbx);
        state.copy(a, a + 3);
    }
}
