//! nzint: Non-zero 64-bit signed integer
//! Invariants:
//! - Value is always non-zero (i64 != 0)
//! - Arithmetic helpers return Result and never construct zero
//! Design choices:
//! - Backed by core::num::NonZeroI64 for niche optimization (zero-cost)

use core::fmt;
use core::hash::{Hash, Hasher};
use core::num::NonZeroI64;

/// Error type for nzint operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NzError {
    /// The result would be zero.
    ZeroResult,
    /// Integer division overflow (e.g., i64::MIN / -1).
    DivOverflow,
}

#[derive(Clone, Copy)]
pub struct NzInt(NonZeroI64);

impl NzInt {
    /// Create a new NzInt. Returns None if v == 0.
    #[inline]
    pub fn new(v: i64) -> Option<Self> {
        NonZeroI64::new(v).map(NzInt)
    }

    /// Create a new NzInt without checking. Caller must guarantee v != 0.
    /// # Safety
    /// Passing 0 is UB for NonZeroI64 and breaks invariants.
    #[inline]
    pub unsafe fn new_unchecked(v: i64) -> Self {
        NzInt(NonZeroI64::new_unchecked(v))
    }

    /// Get the inner i64.
    #[inline]
    pub fn get(self) -> i64 {
        self.0.get()
    }

    /// Checked addition. Returns Err(ZeroResult) if the sum is zero.
    #[inline]
    pub fn checked_add(self, rhs: NzInt) -> Result<NzInt, NzError> {
        let a = self.get();
        let b = rhs.get();
        let (res, overflow) = a.overflowing_add(b);
        if overflow {
            // Overflow can never yield 0 for i64 unless wrapping hits 0 exactly.
            // Guard anyway using the invariant below.
            if res == 0 {
                return Err(NzError::ZeroResult);
            }
            // Non-zero and overflowed -> still a valid i64; construct via NonZeroI64::new_unchecked.
            return Ok(unsafe { NzInt::new_unchecked(res) });
        }
        if res == 0 {
            Err(NzError::ZeroResult)
        } else {
            Ok(unsafe { NzInt::new_unchecked(res) })
        }
    }

    /// Checked subtraction. Returns Err(ZeroResult) if the difference is zero.
    #[inline]
    pub fn checked_sub(self, rhs: NzInt) -> Result<NzInt, NzError> {
        let a = self.get();
        let b = rhs.get();
        let (res, overflow) = a.overflowing_sub(b);
        if overflow {
            if res == 0 {
                return Err(NzError::ZeroResult);
            }
            return Ok(unsafe { NzInt::new_unchecked(res) });
        }
        if res == 0 {
            Err(NzError::ZeroResult)
        } else {
            Ok(unsafe { NzInt::new_unchecked(res) })
        }
    }

    /// Checked multiplication. Returns Err(ZeroResult) if the product is zero.
    #[inline]
    pub fn checked_mul(self, rhs: NzInt) -> Result<NzInt, NzError> {
        let a = self.get();
        let b = rhs.get();
        // If either factor is +/-1, product can be zero only if the other is 0 (which cannot happen).
        // For general case use overflowing_mul and check result.
        let (res, overflow) = a.overflowing_mul(b);
        if overflow {
            if res == 0 {
                return Err(NzError::ZeroResult);
            }
            return Ok(unsafe { NzInt::new_unchecked(res) });
        }
        if res == 0 {
            Err(NzError::ZeroResult)
        } else {
            Ok(unsafe { NzInt::new_unchecked(res) })
        }
    }

    /// Checked division (truncates toward zero).
    /// Returns:
    /// - Err(ZeroResult) if quotient is zero.
    /// - Err(DivOverflow) if a == i64::MIN and b == -1 (overflow in two's complement).
    #[inline]
    pub fn checked_div(self, rhs: NzInt) -> Result<NzInt, NzError> {
        let a = self.get();
        let b = rhs.get();
        // Divisor is guaranteed non-zero by invariant.
        if a == i64::MIN && b == -1 {
            // i64::MIN / -1 overflows
            return Err(NzError::DivOverflow);
        }
        let q = a / b;
        if q == 0 {
            Err(NzError::ZeroResult)
        } else {
            Ok(unsafe { NzInt::new_unchecked(q) })
        }
    }

    /// Checked negation. Returns Err(ZeroResult) if result would be zero (impossible for nzint).
    /// Returns Err(DivOverflow) when negating i64::MIN.
    #[inline]
    pub fn checked_neg(self) -> Result<NzInt, NzError> {
        let a = self.get();
        if a == i64::MIN {
            return Err(NzError::DivOverflow);
        }
        let r = -a;
        debug_assert!(r != 0);
        Ok(unsafe { NzInt::new_unchecked(r) })
    }

    /// Absolute value. Returns Err(DivOverflow) for i64::MIN.
    #[inline]
    pub fn checked_abs(self) -> Result<NzInt, NzError> {
        let a = self.get();
        if a == i64::MIN {
            return Err(NzError::DivOverflow);
        }
        let r = a.abs();
        debug_assert!(r != 0);
        Ok(unsafe { NzInt::new_unchecked(r) })
    }

    /// Sign of the value: +1 for positive, -1 for negative (as NzInt).
    #[inline]
    pub fn signum(self) -> NzInt {
        // a != 0 always holds; (a > 0) as i64 yields 0/1, so avoid that.
        if self.get() > 0 {
            unsafe { NzInt::new_unchecked(1) }
        } else {
            unsafe { NzInt::new_unchecked(-1) }
        }
    }
}

/* ----- Trait impls (Copy/Clone/Eq/Ord/Hash/Display/Debug/TryFrom/From) ----- */

impl fmt::Debug for NzInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NzInt").field(&self.get()).finish()
    }
}

impl fmt::Display for NzInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl PartialEq for NzInt {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
impl Eq for NzInt {}

impl PartialOrd for NzInt {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NzInt {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

impl Hash for NzInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl From<NonZeroI64> for NzInt {
    #[inline]
    fn from(nz: NonZeroI64) -> Self {
        NzInt(nz)
    }
}

impl TryFrom<i64> for NzInt {
    type Error = NzError;
    #[inline]
    fn try_from(v: i64) -> Result<Self, Self::Error> {
        NzInt::new(v).ok_or(NzError::ZeroResult)
    }
}

/* ----- Optional convenience constructors for small non-zero constants ----- */

impl NzInt {
    /// Construct +1.
    #[inline]
    pub fn one() -> Self {
        unsafe { NzInt::new_unchecked(1) }
    }
    /// Construct -1.
    #[inline]
    pub fn neg_one() -> Self {
        unsafe { NzInt::new_unchecked(-1) }
    }
}
