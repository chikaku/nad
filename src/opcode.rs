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
}

macro_rules! code {
    ($test:expr, $seta:expr, $argb:ident, $argc:ident, $mode:ident, $name:expr) => {
        Code {
            test_flag: $test,
            seta_flag: $seta,
            argb_mode: ArgType::$argb,
            argc_mode: ArgType::$argc,

            op_mode: Mode::$mode,
            name: $name,
        }
    };
}

pub const ALL: &'static [Code] = &[
    /*    T  A  B  C  mode         name    */
    code!(0, 1, R, N, IABC /* */, "MOVE    "), // R(A) := R(B)
    code!(0, 1, K, N, IABx /* */, "LOADK   "), // R(A) := Kst(Bx)
    code!(0, 1, N, N, IABx /* */, "LOADKX  "), // R(A) := Kst(extra arg)
    code!(0, 1, U, U, IABC /* */, "LOADBOOL"), // R(A) := (bool)B; if (C) pc++
    code!(0, 1, U, N, IABC /* */, "LOADNIL "), // R(A), R(A+1), ..., R(A+B) := nil
    code!(0, 1, U, N, IABC /* */, "GETUPVAL"), // R(A) := UpValue[B]
    code!(0, 1, U, K, IABC /* */, "GETTABUP"), // R(A) := UpValue[B][RK(C)]
    code!(0, 1, R, K, IABC /* */, "GETTABLE"), // R(A) := R(B)[RK(C)]
    code!(0, 0, K, K, IABC /* */, "SETTABUP"), // UpValue[A][RK(B)] := RK(C)
    code!(0, 0, U, N, IABC /* */, "SETUPVAL"), // UpValue[B] := R(A)
    code!(0, 0, K, K, IABC /* */, "SETTABLE"), // R(A)[RK(B)] := RK(C)
    code!(0, 1, U, U, IABC /* */, "NEWTABLE"), // R(A) := {} (size = B,C)
    code!(0, 1, R, K, IABC /* */, "SELF    "), // R(A+1) := R(B); R(A) := R(B)[RK(C)]
    code!(0, 1, K, K, IABC /* */, "ADD     "), // R(A) := RK(B) + RK(C)
    code!(0, 1, K, K, IABC /* */, "SUB     "), // R(A) := RK(B) - RK(C)
    code!(0, 1, K, K, IABC /* */, "MUL     "), // R(A) := RK(B) * RK(C)
    code!(0, 1, K, K, IABC /* */, "MOD     "), // R(A) := RK(B) % RK(C)
    code!(0, 1, K, K, IABC /* */, "POW     "), // R(A) := RK(B) ^ RK(C)
    code!(0, 1, K, K, IABC /* */, "DIV     "), // R(A) := RK(B) / RK(C)
    code!(0, 1, K, K, IABC /* */, "IDIV    "), // R(A) := RK(B) // RK(C)
    code!(0, 1, K, K, IABC /* */, "BAND    "), // R(A) := RK(B) & RK(C)
    code!(0, 1, K, K, IABC /* */, "BOR     "), // R(A) := RK(B) | RK(C)
    code!(0, 1, K, K, IABC /* */, "BXOR    "), // R(A) := RK(B) ~ RK(C)
    code!(0, 1, K, K, IABC /* */, "SHL     "), // R(A) := RK(B) << RK(C)
    code!(0, 1, K, K, IABC /* */, "SHR     "), // R(A) := RK(B) >> RK(C)
    code!(0, 1, R, N, IABC /* */, "UNM     "), // R(A) := -R(B)
    code!(0, 1, R, N, IABC /* */, "BNOT    "), // R(A) := ~R(B)
    code!(0, 1, R, N, IABC /* */, "NOT     "), // R(A) := not R(B)
    code!(0, 1, R, N, IABC /* */, "LEN     "), // R(A) := length of R(B)
    code!(0, 1, R, R, IABC /* */, "CONCAT  "), // R(A) := R(B).. ... ..R(C)
    code!(0, 0, R, N, IAsBx /**/, "JMP     "), // pc+=sBx; if (A) close all upvalues >= R(A - 1)
    code!(1, 0, K, K, IABC /* */, "EQ      "), // if ((RK(B) == RK(C)) ~= A) then pc++
    code!(1, 0, K, K, IABC /* */, "LT      "), // if ((RK(B) <  RK(C)) ~= A) then pc++
    code!(1, 0, K, K, IABC /* */, "LE      "), // if ((RK(B) <= RK(C)) ~= A) then pc++
    code!(1, 0, N, U, IABC /* */, "TEST    "), // if not (R(A) <=> C) then pc++
    code!(1, 1, R, U, IABC /* */, "TESTSET "), // if (R(B) <=> C) then R(A) := R(B) else pc++
    code!(0, 1, U, U, IABC /* */, "CALL    "), // R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
    code!(0, 1, U, U, IABC /* */, "TAILCALL"), // return R(A)(R(A+1), ... ,R(A+B-1))
    code!(0, 0, U, N, IABC /* */, "RETURN  "), // return R(A), ... ,R(A+B-2)
    code!(0, 1, R, N, IAsBx /**/, "FORLOOP "), // R(A)+=R(A+2); if R(A) <?= R(A+1) then { pc+=sBx; R(A+3)=R(A) }
    code!(0, 1, R, N, IAsBx /**/, "FORPREP "), // R(A)-=R(A+2); pc+=sBx
    code!(0, 0, N, U, IABC /* */, "TFORCALL"), // R(A+3), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
    code!(0, 1, R, N, IAsBx /**/, "TFORLOOP"), // if R(A+1) ~= nil then { R(A)=R(A+1); pc += sBx }
    code!(0, 0, U, U, IABC /* */, "SETLIST "), // R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    code!(0, 1, U, N, IABx /* */, "CLOSURE "), // R(A) := closure(KPROTO[Bx])
    code!(0, 1, U, N, IABC /* */, "VARARG  "), // R(A), R(A+1), ..., R(A+B-2) = vararg
    code!(0, 0, U, U, IAx /*  */, "EXTRAARG"), // extra (larger) argument for previous opcode
];
