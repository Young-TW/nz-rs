//! nzfloat: Non-zero, non-NaN 64-bit float
//! Invariants:
//! - Value is finite or infinite, but never 0.0, -0.0, or NaN
//! API:
//! - NzFloat::new(v) -> Option<Self>
//! - get(), checked_add/sub/mul/div, abs(), signum()
//! - TryFrom<f64>, Display/Debug/Ord/Hash

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NzfError {
    ZeroResult,     // result is 0.0 or -0.0
    NotANumber,     // NaN encountered
}

#[derive(Clone, Copy)]
pub struct NzFloat(f64);

impl NzFloat {
    /// Create from f64; rejects 0.0, -0.0, NaN.
    #[inline]
    pub fn new(v: f64) -> Option<Self> {
        if v == 0.0 || v.is_nan() { None } else { Some(NzFloat(v)) }
    }

    /// Create without checks. Caller must ensure v != 0.0 and !NaN.
    /// # Safety
    /// Passing 0.0/-0.0/NaN breaks invariants.
    #[inline]
    pub unsafe fn new_unchecked(v: f64) -> Self {
        NzFloat(v)
    }

    /// Get inner f64.
    #[inline]
    pub fn get(self) -> f64 {
        self.0
    }

    /// Checked addition.
    #[inline]
    pub fn checked_add(self, rhs: NzFloat) -> Result<NzFloat, NzfError> {
        let r = self.0 + rhs.0;
        if r.is_nan() { return Err(NzfError::NotANumber); }
        if r == 0.0 { return Err(NzfError::ZeroResult); }
        Ok(unsafe { NzFloat::new_unchecked(r) })
    }

    /// Checked subtraction.
    #[inline]
    pub fn checked_sub(self, rhs: NzFloat) -> Result<NzFloat, NzfError> {
        let r = self.0 - rhs.0;
        if r.is_nan() { return Err(NzfError::NotANumber); }
        if r == 0.0 { return Err(NzfError::ZeroResult); }
        Ok(unsafe { NzFloat::new_unchecked(r) })
    }

    /// Checked multiplication.
    #[inline]
    pub fn checked_mul(self, rhs: NzFloat) -> Result<NzFloat, NzfError> {
        let r = self.0 * rhs.0;
        if r.is_nan() { return Err(NzfError::NotANumber); }
        if r == 0.0 { return Err(NzfError::ZeroResult); }
        Ok(unsafe { NzFloat::new_unchecked(r) })
    }

    /// Checked division (IEEE-754, allows ±inf).
    #[inline]
    pub fn checked_div(self, rhs: NzFloat) -> Result<NzFloat, NzfError> {
        // rhs is guaranteed non-zero by invariant
        let r = self.0 / rhs.0;
        if r.is_nan() { return Err(NzfError::NotANumber); }
        if r == 0.0 { return Err(NzfError::ZeroResult); }
        Ok(unsafe { NzFloat::new_unchecked(r) })
    }

    /// Absolute value.
    #[inline]
    pub fn abs(self) -> NzFloat {
        // abs(x) != 0.0 because x != 0.0
        let r = self.0.abs();
        debug_assert!(r != 0.0 && !r.is_nan());
        unsafe { NzFloat::new_unchecked(r) }
    }

    /// Sign as ±1.0 (non-zero).
    #[inline]
    pub fn signum(self) -> NzFloat {
        if self.0.is_sign_positive() {
            unsafe { NzFloat::new_unchecked(1.0) }
        } else {
            unsafe { NzFloat::new_unchecked(-1.0) }
        }
    }

    /// Construct +1.0.
    #[inline]
    pub fn one() -> NzFloat {
        unsafe { NzFloat::new_unchecked(1.0) }
    }

    /// Construct -1.0.
    #[inline]
    pub fn neg_one() -> NzFloat {
        unsafe { NzFloat::new_unchecked(-1.0) }
    }
}

/* ----- Trait impls ----- */

impl fmt::Debug for NzFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NzFloat").field(&self.0).finish()
    }
}

impl fmt::Display for NzFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Avoid printing -0; invariant ensures not possible
        write!(f, "{}", self.0)
    }
}

impl PartialEq for NzFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for NzFloat {}

impl PartialOrd for NzFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NzFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        // No NaN in domain -> total_cmp is a strict total order
        self.0.total_cmp(&other.0)
    }
}

impl Hash for NzFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // No NaN and no ±0.0 -> to_bits is stable
        self.0.to_bits().hash(state)
    }
}

impl TryFrom<f64> for NzFloat {
    type Error = NzfError;
    #[inline]
    fn try_from(v: f64) -> Result<Self, Self::Error> {
        NzFloat::new(v).ok_or(NzfError::ZeroResult)
    }
}

impl From<NzFloat> for f64 {
    #[inline]
    fn from(v: NzFloat) -> f64 {
        v.0
    }
}
