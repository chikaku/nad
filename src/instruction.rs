use std::fmt;
use std::fmt::Formatter;
use std::ops::BitAnd;

use crate::opcode::{ArgType, Code, Mode, ALL, RET};
use crate::State;

#[derive(Copy, Clone, Hash)]
pub struct Instruction(pub u32);

impl BitAnd<u32> for Instruction {
    type Output = Instruction;

    fn bitand(self, rhs: u32) -> Self::Output {
        Instruction(self.0 & rhs)
    }
}

const MAX_BX: i32 = (1 << 18) - 1;
const MAX_SBX: i32 = MAX_BX >> 1;

impl Instruction {
    pub fn exec(&self, state: &mut State) {
        (&ALL[(self.0 & 0x3F) as usize].exec)(*self, state)
    }

    pub fn opcode(self) -> &'static Code {
        &ALL[(self.0 & 0x3F) as usize]
    }

    pub fn is_ret(&self) -> bool {
        self.0 & 0x3F == RET
    }

    pub fn ax(self) -> i32 {
        (self.0 >> 6) as i32
    }

    pub fn abx(self) -> (i32, i32) {
        (((self.0 >> 6) & 0xFF) as i32, (self.0 >> 14) as i32)
    }

    pub fn abc(self) -> (i32, i32, i32) {
        (
            (self.0 >> 6 & 0xFF) as i32,
            (self.0 >> 23 & 0x1FF) as i32,
            (self.0 >> 14 & 0x1FF) as i32,
        )
    }

    pub fn asbx(self) -> (i32, i32) {
        let (a, bx) = self.abx();
        (a, bx - MAX_SBX)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let code = self.opcode();
        match code.op_mode {
            Mode::IABC => {
                let (a, b, c) = self.abc();
                write!(
                    f,
                    "{} {} {} {}",
                    code.name,
                    a,
                    if code.argb_mode == ArgType::N {
                        "".to_string()
                    } else {
                        format!("{}", if b > 0xFF { -1 - (b & 0xFF) } else { b })
                    },
                    if code.argc_mode == ArgType::N {
                        "".to_string()
                    } else {
                        format!("{}", if c > 0xFF { -1 - (c & 0xFF) } else { c })
                    },
                )
            }
            Mode::IABx => {
                let (a, bx) = self.abx();
                write!(
                    f,
                    "{} {} {}",
                    code.name,
                    a,
                    match code.argb_mode {
                        ArgType::K => -1 - bx,
                        ArgType::U => bx,
                        _ => unreachable!(),
                    }
                )
            }
            Mode::IAsBx => {
                let (a, sbx) = self.asbx();
                write!(f, "{} {} {}", code.name, a, sbx)
            }
            Mode::IAx => {
                write!(f, "{} {}", code.name, -1 - self.ax())
            }
        }
    }
}
