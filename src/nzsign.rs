#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum nzSign {
    Neg = -1, // represents false
    Pos =  1, // represents true
}

impl nzSign {
    #[inline] pub fn is_true(self) -> bool { matches!(self, nzSign::Pos) }
    #[inline] pub fn is_false(self) -> bool { matches!(self, nzSign::Neg) }

    // Logical NOT (stay in ±1 domain)
    #[inline] pub fn not(self) -> Self { if self.is_true() { nzSign::Neg } else { nzSign::Pos } }

    // AND/OR implemented as min/max semantics; short-circuiting should be handled at VM instruction level
    #[inline] pub fn and(self, rhs: nzSign) -> nzSign {
        // truth table: (Pos,Pos) -> Pos; otherwise Neg
        if self.is_false() { nzSign::Neg } else { rhs }
    }
    #[inline] pub fn or(self, rhs: nzSign) -> nzSign {
        // truth table: (Neg,Neg) -> Neg; otherwise Pos
        if self.is_true() { nzSign::Pos } else { rhs }
    }

    // XOR (provided only if needed)
    #[inline] pub fn xor(self, rhs: nzSign) -> nzSign {
        if self == rhs { nzSign::Neg } else { nzSign::Pos }
    }

    // Conversion to/from i8/i64 (for serialization/FFI)
    #[inline] pub fn to_i8(self) -> i8 { self as i8 }
    #[inline] pub fn from_i8(v: i8) -> Option<nzSign> {
        match v { 1 => Some(nzSign::Pos), -1 => Some(nzSign::Neg), _ => None }
    }

    // Conversion to/from Rust bool (for host interop)
    #[inline] pub fn to_bool(self) -> bool { self.is_true() }
    #[inline] pub fn from_bool(b: bool) -> Self { if b { nzSign::Pos } else { nzSign::Neg } }
}
