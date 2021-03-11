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

macro_rules! code {
    ($test:expr, $seta:expr, $argb:ident, $argc:ident, $mode:ident, $name:expr, $exec:ident) => {
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
    code!(0, 1, N, N, IABx /* */, "LOADKX  ", load_const_index), // R(A) := Kst(extra arg)
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
    code!(0, 1, K, K, IABC /* */, "ADD     ", unimplement), // R(A) := RK(B) + RK(C)
    code!(0, 1, K, K, IABC /* */, "SUB     ", unimplement), // R(A) := RK(B) - RK(C)
    code!(0, 1, K, K, IABC /* */, "MUL     ", unimplement), // R(A) := RK(B) * RK(C)
    code!(0, 1, K, K, IABC /* */, "MOD     ", unimplement), // R(A) := RK(B) % RK(C)
    code!(0, 1, K, K, IABC /* */, "POW     ", unimplement), // R(A) := RK(B) ^ RK(C)
    code!(0, 1, K, K, IABC /* */, "DIV     ", unimplement), // R(A) := RK(B) / RK(C)
    code!(0, 1, K, K, IABC /* */, "IDIV    ", unimplement), // R(A) := RK(B) // RK(C)
    code!(0, 1, K, K, IABC /* */, "BAND    ", unimplement), // R(A) := RK(B) & RK(C)
    code!(0, 1, K, K, IABC /* */, "BOR     ", unimplement), // R(A) := RK(B) | RK(C)
    code!(0, 1, K, K, IABC /* */, "BXOR    ", unimplement), // R(A) := RK(B) ~ RK(C)
    code!(0, 1, K, K, IABC /* */, "SHL     ", unimplement), // R(A) := RK(B) << RK(C)
    code!(0, 1, K, K, IABC /* */, "SHR     ", unimplement), // R(A) := RK(B) >> RK(C)
    code!(0, 1, R, N, IABC /* */, "UNM     ", unimplement), // R(A) := -R(B)
    code!(0, 1, R, N, IABC /* */, "BNOT    ", unimplement), // R(A) := ~R(B)
    code!(0, 1, R, N, IABC /* */, "NOT     ", unimplement), // R(A) := not R(B)
    code!(0, 1, R, N, IABC /* */, "LEN     ", unimplement), // R(A) := length of R(B)
    code!(0, 1, R, R, IABC /* */, "CONCAT  ", unimplement), // R(A) := R(B).. ... ..R(C)
    code!(0, 0, R, N, IAsBx /**/, "JMP     ", jmp /*   */), // pc+=sBx; if (A) close all upvalues >= R(A - 1)
    code!(1, 0, K, K, IABC /* */, "EQ      ", unimplement), // if ((RK(B) == RK(C)) ~= A) then pc++
    code!(1, 0, K, K, IABC /* */, "LT      ", unimplement), // if ((RK(B) <  RK(C)) ~= A) then pc++
    code!(1, 0, K, K, IABC /* */, "LE      ", unimplement), // if ((RK(B) <= RK(C)) ~= A) then pc++
    code!(1, 0, N, U, IABC /* */, "TEST    ", unimplement), // if not (R(A) <=> C) then pc++
    code!(1, 1, R, U, IABC /* */, "TESTSET ", unimplement), // if (R(B) <=> C) then R(A) := R(B) else pc++
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

fn unimplement(ins: Instruction, state: &mut State) {
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

fn load_const(ins: Instruction, state: &mut State) {
    let (a, bx) = ins.abx();
    assert!(bx >= 0);
    state.get_const(bx as usize);
    state.replace(a + 1);
}

fn load_const_index(ins: Instruction, state: &mut State) {
    let (a, _) = ins.abx();
    let ax = state.fetch().ax();
    assert!(ax >= 0);
    state.get_const(ax as usize);
    state.replace(a);
}
