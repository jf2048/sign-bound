//! # sign-bound
//!
//! Signed integer types for Rust that are bounded to be either positive or negative. The API is
//! analogous to the built-in [`NonZero`] types. The types are all memory-layout optimized, so for
//! example `Option<PositiveI32>` and `Option<NegativeI32>` are both the same size as `i32`. Using
//! additional variants in an enum can also have some space benefits.
//!
//! ```rust
//! # use sign_bound::PositiveI16;
//! enum MyEnum {
//!     A(PositiveI16),
//!     B,
//!     C,
//!     D,
//! }
//! assert_eq!(size_of::<MyEnum>(), size_of::<PositiveI16>());
//! ```
//!
//! Note that due to the implementation details of this crate, the space
//! optimization for any type will not occur if there are more than 128 additional
//! enum variants.
//!
//! `Option<PositiveIsize>` is particularly useful as a space-efficient optional
//! `Vec` index, since Rust's `Vec` structure is
//! [limited](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push) to
//! `isize::MAX` entries.
//!
//! [`NonZero`]: (https://doc.rust-lang.org/std/num/struct.NonZero.html)

#![deny(missing_docs)]
#![no_std]

macro_rules! impl_positive {
    ($(#[$attr:meta])* $ty:ident, $sty:ident, $base:ty, $uns:ty) => {
        /// A signed value that is known to be positive.
        ///
        /// This enables some memory layout optimization.
        #[doc = concat!("For example, `Option<", stringify!($ty), ">` is the same size as [`", stringify!($base), "`].")]
        #[derive(Copy, Clone)]
        #[repr(C)]
        $(#[$attr])*
        pub struct $ty {
            #[cfg(target_endian = "big")]
            _hi: PositiveHighByte,
            _buf: [u8; size_of::<$base>() - 1],
            #[cfg(target_endian = "little")]
            _hi: PositiveHighByte,
        }

        impl $ty {
            /// The size of this positive integer type in bits.
            ///
            #[doc = concat!("This value is equal to [`", stringify!($base), "::BITS`].")]
            pub const BITS: u32 = <$base>::BITS;
            /// The smallest value that can be represented by this positive integer type, 0.
            pub const MIN: Self = unsafe { $ty::new_unchecked(0) };
            #[doc = concat!("The largest value that can be represented by this positive integer type, equal to [`", stringify!($base), "::MAX`].")]
            pub const MAX: Self = unsafe { $ty::new_unchecked(<$base>::MAX) };
            #[doc = concat!("Creates a `", stringify!($ty), "` if the given value is positive.")]
            pub const fn new(value: $base) -> Option<Self> {
                if value < 0 {
                    return None;
                }
                unsafe { Some(core::mem::transmute::<$base, Self>(value)) }
            }
            #[doc = concat!("Creates a `", stringify!($ty), "` without checking whether the value is positive.")]
            /// This results in undefined behaviour if the value is negative.
            ///
            /// # Safety
            ///
            /// The value must not be negative.
            #[inline]
            pub const unsafe fn new_unchecked(value: $base) -> Self {
                debug_assert!(value >= 0);
                core::mem::transmute::<$base, Self>(value)
            }
            /// Returns the contained value as a primitive type.
            #[inline]
            pub const fn get(self) -> $base {
                unsafe {
                    let n = core::mem::transmute::<Self, $base>(self);
                    core::hint::assert_unchecked(n >= 0);
                    n
                }
            }
            /// Returns the number of zeros in the binary representation of `self`.
            #[inline]
            pub const fn count_zeros(self) -> u32 {
                self.get().count_zeros()
            }
            /// Returns the number of ones in the binary representation of `self`.
            #[inline]
            pub const fn count_ones(self) -> u32 {
                self.get().count_ones()
            }
            /// Returns the number of leading zeros in the binary representation of `self`.
            #[inline]
            pub const fn leading_zeros(self) -> u32 {
                self.get().leading_zeros()
            }
            /// Returns the number of trailing zeros in the binary representation of `self`.
            #[inline]
            pub const fn trailing_zeros(self) -> u32 {
                self.get().trailing_zeros()
            }
            /// Returns `true` if and only if `self == (1 << k)` for some `k`.
            #[inline]
            pub const fn is_power_of_two(self) -> bool {
                (self.get() as $uns).is_power_of_two()
            }
            /// Returns the base 2 logarithm of the number, rounded down.
            ///
            /// # Panics
            ///
            /// This function will panic if `self` is zero.
            #[inline]
            pub const fn ilog2(self) -> u32 {
                self.get().ilog2()
            }
            /// Returns the base 10 logarithm of the number, rounded down.
            ///
            /// # Panics
            ///
            /// This function will panic if `self` is zero.
            #[inline]
            pub const fn ilog10(self) -> u32 {
                self.get().ilog10()
            }
            /// Checked negation. Computes `-self`, returning `None` if `self == 0`.
            #[inline]
            pub const fn checked_neg(self) -> Option<$sty> {
                $sty::new(-self.get())
            }
            /// Checked addition. Adds a positive integer to another positive integer.
            /// Checks for overflow and returns [`None`] on overflow.
            /// As a consequence, the result cannot wrap to a negative integer.
            #[inline]
            pub const fn checked_add(self, rhs: Self) -> Option<Self> {
                match self.get().checked_add(rhs.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked subtraction. Subtracts a positive integer from another positive integer.
            /// Returns [`None`] if the result would overflow into a negative integer.
            #[inline]
            pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
                Self::new(self.get() - rhs.get())
            }
            /// Checked multiplication.
            /// Multiplies a positive integer by another positive integer, returning a positive result.
            /// Returns [`None`] if the result would overflow.
            #[inline]
            pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
                match self.get().checked_mul(rhs.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked division.
            /// Divides a positive integer by another positive integer, returning the positive quotient.
            /// Returns [`None`] if `rhs == 0`.
            #[inline]
            pub const fn checked_div(self, rhs: Self) -> Option<Self> {
                match self.get().checked_div(rhs.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked remainder.
            /// Divides a positive integer by another positive integer, returning the positive
            /// remainder.
            /// Returns [`None`] if `rhs == 0`.
            #[inline]
            pub const fn checked_rem(self, rhs: Self) -> Option<Self> {
                match self.get().checked_rem(rhs.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked division by unsigned.
            /// Divides a positive integer by an unsigned integer, returning the positive quotient.
            /// Returns [`None`] if `rhs == 0`.
            #[inline]
            pub const fn checked_div_unsigned(self, rhs: $uns) -> Option<Self> {
                match (self.get() as $uns).checked_div(rhs) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n as $base)) },
                    None => None,
                }
            }
            /// Checked remainder of unsigned.
            /// Divides a positive integer by an unsigned integer, returning the positive remainder.
            /// Returns [`None`] if `rhs == 0`.
            #[inline]
            pub const fn checked_rem_unsigned(self, rhs: $uns) -> Option<Self> {
                match (self.get() as $uns).checked_rem(rhs) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n as $base)) },
                    None => None,
                }
            }
            /// Checked integer exponentiation.
            /// Raises positive value to an integer power.
            /// Checks for overflow and returns [`None`] on overflow.
            /// As a consequence, the result cannot wrap to a negative integer.
            #[inline]
            pub const fn checked_pow(self, rhs: u32) -> Option<Self> {
                match self.get().checked_pow(rhs) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Returns the smallest power of two greater than or equal to `self`.
            /// Checks for overflow and returns [`None`]
            /// if the next power of two is greater than the type’s maximum value.
            /// As a consequence, the result cannot wrap to a negative integer.
            #[inline]
            pub const fn checked_next_power_of_two(self) -> Option<Self> {
                Self::new((self.get() as $uns).next_power_of_two() as $base)
            }
            /// Returns the base 2 logarithm of the number, rounded down.
            ///
            /// Returns `None` if the number is zero.
            #[inline]
            pub const fn checked_ilog2(self) -> Option<u32> {
                self.get().checked_ilog2()
            }
            /// Returns the base 10 logarithm of the number, rounded down.
            ///
            /// Returns `None` if the number is zero.
            #[inline]
            pub const fn checked_ilog10(self) -> Option<u32> {
                self.get().checked_ilog10()
            }
            /// Saturating addition. Adds a positive integer to another positive integer.
            #[doc = concat!("Returns [`", stringify!($ty), "::MAX`] on overflow.")]
            #[inline]
            pub const fn saturating_add(self, rhs: Self) -> Self {
                let n = self.get().saturating_add(rhs.get());
                unsafe { Self::new_unchecked(n) }
            }
            /// Saturating subtraction. Subtracts a positive integer from another positive integer.
            /// Returns 0 if the result would overflow into a negative integer.
            #[inline]
            pub const fn saturating_sub(self, rhs: Self) -> Self {
                match Self::new(self.get() - rhs.get()) {
                    Some(n) => n,
                    None => Self::MIN
                }
            }
            /// Saturating multiplication.
            /// Multiplies a positive integer by another positive integer, returning a positive result.
            #[doc = concat!("Returns [`", stringify!($ty), "::MAX`] on overflow.")]
            #[inline]
            pub const fn saturating_mul(self, rhs: Self) -> Self {
                let n = self.get().saturating_mul(rhs.get());
                unsafe { Self::new_unchecked(n) }
            }
            /// Saturating integer exponentiation.
            /// Raises positive value to an integer power.
            #[doc = concat!("Returns [`", stringify!($ty), "::MAX`] on overflow.")]
            #[inline]
            pub const fn saturating_pow(self, rhs: u32) -> Self {
                let n = self.get().saturating_pow(rhs);
                unsafe { Self::new_unchecked(n) }
            }
        }

        impl Default for $ty {
            #[inline]
            fn default() -> Self {
                Self::MIN
            }
        }

        impl PartialEq for $ty {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.get().eq(&rhs.get())
            }
        }

        impl PartialOrd for $ty {
            fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(rhs))
            }
        }

        impl Ord for $ty {
            fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
                self.get().cmp(&rhs.get())
            }
        }

        impl Eq for $ty {}

        impl core::str::FromStr for $ty {
            type Err = core::num::IntErrorKind;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let n = s.parse::<$uns>().map_err(|e| e.kind().clone())?;
                Self::new(n as $base).ok_or_else(|| core::num::IntErrorKind::PosOverflow)
            }
        }

        impl core::hash::Hash for $ty {
            #[inline]
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.get().hash(state);
            }
        }

        impl core::ops::Div for $ty {
            type Output = Self;
            fn div(self, rhs: Self) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().div(rhs.get())) }
            }
        }
        impl core::ops::DivAssign for $ty {
            fn div_assign(&mut self, rhs: Self) {
                *self = core::ops::Div::div(*self, rhs);
            }
        }

        impl core::ops::Rem for $ty {
            type Output = Self;
            fn rem(self, rhs: Self) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().rem(rhs.get())) }
            }
        }
        impl core::ops::RemAssign for $ty {
            fn rem_assign(&mut self, rhs: Self) {
                *self = core::ops::Rem::rem(*self, rhs);
            }
        }

        impl core::ops::Div<$uns> for $ty {
            type Output = Self;
            fn div(self, rhs: $uns) -> Self::Output {
                unsafe { Self::new_unchecked((self.get() as $uns).div(rhs) as $base) }
            }
        }
        impl core::ops::DivAssign<$uns> for $ty {
            fn div_assign(&mut self, rhs: $uns) {
                *self = core::ops::Div::div(*self, rhs);
            }
        }

        impl core::ops::Rem<$uns> for $ty {
            type Output = Self;
            fn rem(self, rhs: $uns) -> Self::Output {
                unsafe { Self::new_unchecked((self.get() as $uns).rem(rhs) as $base) }
            }
        }
        impl core::ops::RemAssign<$uns> for $ty {
            fn rem_assign(&mut self, rhs: $uns) {
                *self = core::ops::Rem::rem(*self, rhs);
            }
        }

        impl core::ops::BitAnd<$base> for $ty {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: $base) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().bitand(rhs)) }
            }
        }
        impl core::ops::BitAndAssign<$base> for $ty {
            #[inline]
            fn bitand_assign(&mut self, rhs: $base) {
                *self = core::ops::BitAnd::bitand(*self, rhs);
            }
        }

        impl core::ops::BitAnd<$ty> for $base {
            type Output = $ty;
            #[inline]
            fn bitand(self, rhs: $ty) -> Self::Output {
                unsafe { $ty::new_unchecked(self.bitand(rhs.get())) }
            }
        }

        impl_bit_op! { BitOr::bitor, BitOrAssign::bitor_assign for $ty }
        impl_bit_op! { BitAnd::bitand, BitAndAssign::bitand_assign for $ty }
        impl_bit_op! { BitXor::bitxor, BitXorAssign::bitxor_assign for $ty }
        impl_fmt! { Display, Debug, Binary, Octal, LowerHex, UpperHex => $ty }
    };
}

macro_rules! impl_negative {
    ($(#[$attr:meta])* $ty:ident, $pty:ident, $base:ty, $uns:ty) => {
        /// A signed value that is known to be negative.
        ///
        /// This enables some memory layout optimization.
        #[doc = concat!("For example, `Option<", stringify!($ty), ">` is the same size as [`", stringify!($base), "`].")]
        #[derive(Copy, Clone)]
        $(#[$attr])*
        #[repr(C)]
        pub struct $ty {
            #[cfg(target_endian = "big")]
            _hi: NegativeHighByte,
            _buf: [u8; size_of::<$base>() - 1],
            #[cfg(target_endian = "little")]
            _hi: NegativeHighByte,
        }

        impl $ty {
            /// The size of this negative integer type in bits.
            ///
            #[doc = concat!("This value is equal to [`", stringify!($base), "::BITS`].")]
            pub const BITS: u32 = <$base>::BITS;
            #[doc = concat!("The smallest value that can be represented by this negative integer type, equal to [`", stringify!($base), "::MIN`].")]
            pub const MIN: Self = unsafe { $ty::new_unchecked(<$base>::MIN) };
            /// The largest value that can be represented by this negative integer type, -1.
            pub const MAX: Self = unsafe { $ty::new_unchecked(-1) };
            #[doc = concat!("Creates a `", stringify!($ty), "` if the given value is negative.")]
            pub const fn new(value: $base) -> Option<Self> {
                if value >= 0 {
                    return None;
                }
                unsafe { Some(core::mem::transmute::<$base, Self>(value)) }
            }
            #[doc = concat!("Creates a `", stringify!($ty), "` without checking whether the value is negative.")]
            /// This results in undefined behaviour if the value is positive.
            ///
            /// # Safety
            ///
            /// The value must not be positive.
            #[inline]
            pub const unsafe fn new_unchecked(value: $base) -> Self {
                debug_assert!(value < 0);
                core::mem::transmute::<$base, Self>(value)
            }
            /// Returns the contained value as a primitive type.
            #[inline]
            pub const fn get(self) -> $base {
                unsafe {
                    let n = core::mem::transmute::<Self, $base>(self);
                    core::hint::assert_unchecked(n < 0);
                    n
                }
            }
            /// Returns the number of zeros in the binary representation of `self`.
            #[inline]
            pub const fn count_zeros(self) -> u32 {
                self.get().count_zeros()
            }
            /// Returns the number of ones in the binary representation of `self`.
            #[inline]
            pub const fn count_ones(self) -> u32 {
                self.get().count_ones()
            }
            /// Returns the number of leading zeros in the binary representation of `self`.
            ///
            /// Since the value is guaranteed to be negative, this function always returns 0.
            #[inline]
            pub const fn leading_zeros(self) -> u32 {
                0
            }
            /// Returns the number of trailing zeros in the binary representation of `self`.
            ///
            /// On many architectures, this function can perform better than `trailing_zeros()` on
            /// the underlying integer type, as special handling of zero can be avoided.
            #[inline]
            pub const fn trailing_zeros(self) -> u32 {
                self.get().trailing_zeros()
            }
            /// Checked absolute value.
            /// Computes `-self`, returning [`None`] if <code>self == [MIN][Self::MIN]</code>.
            #[inline]
            pub const fn checked_abs(self) -> Option<$pty> {
                match self.get().checked_abs() {
                    Some(n) => unsafe { Some($pty::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked negation.
            /// Computes `-self`, returning [`None`] if <code>self == [MIN][Self::MIN]</code>.
            #[inline]
            pub const fn checked_neg(self) -> Option<$pty> {
                match self.get().checked_neg() {
                    Some(n) => unsafe { Some($pty::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked addition. Adds a negative integer to another negative integer.
            /// Checks for overflow and returns [`None`] on overflow.
            /// As a consequence, the result cannot wrap to positive integers.
            #[inline]
            pub const fn checked_add(self, rhs: Self) -> Option<Self> {
                match self.get().checked_add(rhs.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked subtraction. Subtracts a negative integer from another negative integer.
            /// Returns [`None`] if the result would overflow into a positive integer.
            #[inline]
            pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
                Self::new(self.get() - rhs.get())
            }
            /// Checked multiplication.
            /// Multiplies a negative integer by another negative integer, returning a positive result.
            /// Returns [`None`] if the result would overflow.
            #[inline]
            pub const fn checked_mul(self, rhs: Self) -> Option<$pty> {
                match self.get().checked_mul(rhs.get()) {
                    Some(n) => unsafe { Some($pty::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked sign-preserving multiplication. Multiplies a negative integer by a positive
            /// integer, returning a negative result.
            /// Returns [`None`] if `rhs == 0` or the result would overflow.
            #[inline]
            pub const fn checked_mul_positive(self, rhs: $pty) -> Option<Self> {
                match self.get().checked_mul(rhs.get()) {
                    Some(n) => Self::new(n),
                    None => None,
                }
            }
            /// Checked division.
            /// Divides a negative integer by another negative integer, returning the positive quotient.
            /// Returns [`None`] if the result would overflow.
            ///
            /// The only case where such an overflow can occur is when one divides
            /// <code>[MIN][Self::MIN] / -1</code>; this is equivalent to
            /// <code>-[MIN][Self::MIN]</code>, a positive value that is too large to represent
            #[doc = concat!("as a [`", stringify!($pty), "`].")]
            #[inline]
            pub const fn checked_div(self, rhs: Self) -> Option<$pty> {
                match self.get().checked_div(rhs.get()) {
                    Some(n) => unsafe { Some($pty::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked Euclidean division.
            #[doc = concat!("Calculates the [Euclidean quotient](", stringify!($base), "::div_euclid)")]
            /// of two negative integers, returning the positive result.
            /// Returns [`None`] if the result would overflow.
            ///
            /// The only case where such an overflow can occur is when one divides
            /// <code>[MIN][Self::MIN] / -1</code>; this is equivalent to
            /// <code>-[MIN][Self::MIN]</code>, a positive value that is too large to represent
            #[doc = concat!("as a [`", stringify!($pty), "`].")]
            #[inline]
            pub const fn checked_div_euclid(self, rhs: Self) -> Option<$pty> {
                match self.get().checked_div_euclid(rhs.get()) {
                    Some(n) => unsafe { Some($pty::new_unchecked(n)) },
                    None => None,
                }
            }
            /// Checked Euclidean remainder.
            #[doc = concat!("Calculates the [Euclidean remainder](", stringify!($base), "::rem_euclid)")]
            /// of a negative integer and any signed integer, returning the positive result.
            /// Returns [`None`] if `rhs == 0` or the result would overflow.
            ///
            /// The only case where such an overflow can occur is when one divides
            /// <code>[MIN][Self::MIN] / -1</code>; this is equivalent to
            /// <code>-[MIN][Self::MIN]</code>, a positive value that is too large to represent
            #[doc = concat!("as a [`", stringify!($pty), "`].")]
            #[inline]
            pub const fn checked_rem_euclid(self, rhs: $base) -> Option<$pty> {
                let n = self.get().rem_euclid(rhs);
                unsafe { Some($pty::new_unchecked(n)) }
            }
            /// Saturating absolute value.
            /// Computes `-self`, returning
            #[doc = concat!("[`", stringify!($pty), "::MAX`]")]
            /// if <code>self == [MIN][Self::MIN]</code>.
            #[inline]
            pub const fn saturating_abs(self) -> $pty {
                let n = self.get().saturating_abs();
                unsafe { $pty::new_unchecked(n) }
            }
            /// Saturating negation.
            /// Computes `-self`, returning
            #[doc = concat!("[`", stringify!($pty), "::MAX`]")]
            /// if <code>self == [MIN][Self::MIN]</code>.
            #[inline]
            pub const fn saturating_neg(self) -> $pty {
                let n = self.get().saturating_neg();
                unsafe { $pty::new_unchecked(n) }
            }
            /// Saturating addition. Adds a negative integer to another negative integer.
            #[doc = concat!("Returns [`", stringify!($ty), "::MIN`] on overflow.")]
            #[inline]
            pub const fn saturating_add(self, rhs: Self) -> Self {
                let n = self.get().saturating_add(rhs.get());
                unsafe { Self::new_unchecked(n) }
            }
            /// Saturating subtraction. Subtracts a negative integer from another negative integer.
            /// Returns -1 if the result would overflow into a positive integer.
            #[inline]
            pub const fn saturating_sub(self, rhs: Self) -> Self {
                match Self::new(self.get() - rhs.get()) {
                    Some(n) => n,
                    None => Self::MAX
                }
            }
            /// Saturating multiplication.
            /// Multiplies a negative integer by another negative integer, returning a positive result.
            #[doc = concat!("Returns [`", stringify!($pty), "::MAX`] on overflow.")]
            #[inline]
            pub const fn saturating_mul(self, rhs: Self) -> $pty {
                let n = self.get().saturating_mul(rhs.get());
                unsafe { $pty::new_unchecked(n) }
            }
            /// Saturating sign-preserving multiplication.
            /// Multiplies a negative integer by a positive integer, returning a negative result.
            /// Returns -1 if `rhs == 0`.
            #[doc = concat!("Returns [`", stringify!($ty), "::MIN`] on overflow.")]
            #[inline]
            pub const fn saturating_mul_positive(self, rhs: $pty) -> Self {
                match Self::new(self.get().saturating_mul(rhs.get())) {
                    Some(n) => n,
                    None => Self::MAX,
                }
            }
        }

        impl PartialEq for $ty {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.get().eq(&rhs.get())
            }
        }

        impl PartialOrd for $ty {
            fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
                Some(self.cmp(rhs))
            }
        }

        impl Ord for $ty {
            fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
                self.get().cmp(&rhs.get())
            }
        }

        impl Eq for $ty {}

        impl core::str::FromStr for $ty {
            type Err = core::num::IntErrorKind;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let n = s.parse::<$base>().map_err(|e| e.kind().clone())?;
                Self::new(n).ok_or_else(|| core::num::IntErrorKind::PosOverflow)
            }
        }

        impl core::hash::Hash for $ty {
            #[inline]
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.get().hash(state);
            }
        }

        impl core::ops::BitOr<$base> for $ty {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: $base) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().bitor(rhs)) }
            }
        }
        impl core::ops::BitOrAssign<$base> for $ty {
            #[inline]
            fn bitor_assign(&mut self, rhs: $base) {
                *self = core::ops::BitOr::bitor(*self, rhs);
            }
        }

        impl core::ops::BitOr<$ty> for $base {
            type Output = $ty;
            #[inline]
            fn bitor(self, rhs: $ty) -> Self::Output {
                unsafe { $ty::new_unchecked(self.bitor(rhs.get())) }
            }
        }

        impl core::ops::BitOr<$ty> for $pty {
            type Output = $ty;
            #[inline]
            fn bitor(self, rhs: $ty) -> Self::Output {
                unsafe { $ty::new_unchecked(self.get().bitor(rhs.get())) }
            }
        }
        impl core::ops::BitOr<$pty> for $ty {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: $pty) -> Self::Output {
                unsafe { $ty::new_unchecked(self.get().bitor(rhs.get())) }
            }
        }
        impl core::ops::BitOrAssign<$pty> for $ty {
            #[inline]
            fn bitor_assign(&mut self, rhs: $pty) {
                *self = core::ops::BitOr::bitor(*self, rhs);
            }
        }

        impl core::ops::BitAnd<$pty> for $ty {
            type Output = $pty;
            #[inline]
            fn bitand(self, rhs: $pty) -> Self::Output {
                unsafe { $pty::new_unchecked(self.get().bitand(rhs.get())) }
            }
        }
        impl core::ops::BitAnd<$ty> for $pty {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: $ty) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().bitand(rhs.get())) }
            }
        }
        impl core::ops::BitAndAssign<$ty> for $pty {
            #[inline]
            fn bitand_assign(&mut self, rhs: $ty) {
                *self = core::ops::BitAnd::bitand(*self, rhs);
            }
        }

        impl core::ops::BitXor<$pty> for $ty {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: $pty) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().bitxor(rhs.get())) }
            }
        }
        impl core::ops::BitXorAssign<$pty> for $ty {
            #[inline]
            fn bitxor_assign(&mut self, rhs: $pty) {
                *self = core::ops::BitXor::bitxor(*self, rhs);
            }
        }
        impl core::ops::BitXor<$ty> for $pty {
            type Output = $ty;
            #[inline]
            fn bitxor(self, rhs: $ty) -> Self::Output {
                unsafe { $ty::new_unchecked(self.get().bitxor(rhs.get())) }
            }
        }
        impl core::ops::BitXor for $ty {
            type Output = $pty;
            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                unsafe { $pty::new_unchecked(self.get().bitxor(rhs.get())) }
            }
        }

        impl core::ops::Not for $ty {
            type Output = $pty;
            fn not(self) -> Self::Output {
                unsafe { $pty::new_unchecked(self.get().not()) }
            }
        }
        impl core::ops::Not for $pty {
            type Output = $ty;
            fn not(self) -> Self::Output {
                unsafe { $ty::new_unchecked(self.get().not()) }
            }
        }

        impl_fmt! { Display, Debug, Binary, Octal, LowerHex, UpperHex => $ty }
        impl_bit_op! { BitOr::bitor, BitOrAssign::bitor_assign for $ty }
        impl_bit_op! { BitAnd::bitand, BitAndAssign::bitand_assign for $ty }
    };
}

macro_rules! impl_fmt {
    (=> $ty:ty) => {};
    ($trait:ident $(, $rest:ident)* => $ty:ty) => {
        impl core::fmt::$trait for $ty {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::$trait::fmt(&self.get(), f)
            }
        }
        impl_fmt! { $($rest),* => $ty }
    };
}

macro_rules! impl_from {
    (=> $ty:ty) => {};
    ($from:ty $(, $rest:ty)* => $ty:ty) => {
        impl From<$from> for $ty {
            #[inline]
            fn from(value: $from) -> Self {
                unsafe { Self::new_unchecked(value as _) }
            }
        }
        impl_from! { $($rest),* => $ty }
    };
}

macro_rules! impl_from_get {
    ($ty:ty =>) => {};
    ($ty:ty => $from:ty $(, $rest:ty)*) => {
        impl From<$ty> for $from {
            #[inline]
            fn from(value: $ty) -> Self {
                unsafe { Self::new_unchecked(value.get() as _) }
            }
        }
        impl TryFrom<$from> for $ty {
            type Error = core::num::TryFromIntError;
            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let value = <_>::try_from(value.get())?;
                unsafe { Ok(Self::new_unchecked(value)) }
            }
        }
        impl_from_get! { $ty => $($rest),*}
    };
}

macro_rules! impl_primitive_from {
    ($ty:ty =>) => {};
    ($ty:ty => $from:ty $(, $rest:ty)*) => {
        impl From<$ty> for $from {
            #[inline]
            fn from(value: $ty) -> Self {
                value.get() as _
            }
        }
        impl_primitive_from! { $ty => $($rest),* }
    };
}

macro_rules! impl_try_from {
    ($ty:ty =>) => {};
    ($ty:ty => $from:ty $(, $rest:ty)*) => {
        impl TryFrom<$from> for $ty {
            type Error = core::num::TryFromIntError;
            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let value = <_>::try_from(value.get())?;
                unsafe { Ok(Self::new_unchecked(value)) }
            }
        }
        impl_try_from! { $ty => $($rest),*}
    };
}

macro_rules! impl_primitive_try_from {
    ($ty:ty =>) => {};
    ($ty:ty => $from:ty $(, $rest:ty)*) => {
        impl TryFrom<$ty> for $from {
            type Error = core::num::TryFromIntError;
            #[inline]
            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                Self::try_from(value.get())
            }
        }
        impl_primitive_try_from! { $ty => $($rest),* }
    };
}

macro_rules! impl_positive_try_from {
    (=> $ty:ty $(, $base:ty)* ) => {};
    ($from:ty $(, $rest:ty)* => $ty:ty $(, $base:ty)* ) => {
        impl TryFrom<$from> for $ty {
            type Error = core::num::TryFromIntError;
            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                $(let value = <$base>::try_from(value)?;)*
                unsafe { Ok(Self::new_unchecked(value as _)) }
            }
        }
        impl_positive_try_from! { $($rest),* => $ty $(, $base)* }
    };
}

macro_rules! impl_negative_try_from {
    (=> $ty:ty, $uns:ty, $base:ty) => {};
    ($from:ty $(, $rest:ty)* => $ty:ty, $uns:ty, $base:ty) => {
        impl TryFrom<$from> for $ty {
            type Error = core::num::TryFromIntError;
            #[inline]
            fn try_from(value: $from) -> Result<Self, Self::Error> {
                let value = <$base>::try_from(value)?;
                Self::new(value).ok_or_else(|| <$base>::try_from(<$uns>::MAX).unwrap_err())
            }
        }
        impl_negative_try_from! { $($rest),* => $ty, $uns, $base }
    };
}
macro_rules! impl_bit_op {
    ($op:ident :: $opm:ident, $aop:ident :: $aopm:ident for $ty:ty) => {
        impl core::ops::$op for $ty {
            type Output = Self;
            #[inline]
            fn $opm(self, rhs: Self) -> Self::Output {
                unsafe { Self::new_unchecked(self.get().$opm(rhs.get())) }
            }
        }

        impl core::ops::$aop for $ty {
            #[inline]
            fn $aopm(&mut self, rhs: Self) {
                *self = core::ops::$op::$opm(*self, rhs);
            }
        }
    };
}

impl_positive! { #[repr(align(1))] PositiveI8, NegativeI8, i8, u8 }
impl_from_get! { PositiveI8 => PositiveI16, PositiveI32, PositiveI64, PositiveIsize }
impl_primitive_from! { PositiveI8 => u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize }
impl_positive_try_from! { u8, u16, u32, u64, u128, usize => PositiveI8, i8 }
impl_positive_try_from! { i16, i32, i64, i128, isize => PositiveI8, u8, i8 }
impl_positive_try_from! { i8 => PositiveI8, u8 }
impl_negative! { #[repr(align(1))] NegativeI8, PositiveI8, i8, u8 }
impl_from_get! { NegativeI8 => NegativeI16, NegativeI32, NegativeI64, NegativeIsize }
impl_primitive_from! { NegativeI8 => i8, i16, i32, i64, i128, isize }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI8, u8, i8 }

impl_positive! { #[repr(align(2))] PositiveI16, NegativeI16, i16, u16 }
impl_from! { u8 => PositiveI16 }
impl_from_get! { PositiveI16 => PositiveI32, PositiveI64, PositiveIsize }
impl_primitive_from! { PositiveI16 => u16, u32, u64, u128, usize, i16, i32, i64, i128, isize }
impl_primitive_try_from! { PositiveI16 => u8, i8 }
impl_positive_try_from! { u16, u32, u64, u128, usize => PositiveI16, i16 }
impl_positive_try_from! { i8, i32, i64, i128, isize => PositiveI16, u16, i16 }
impl_positive_try_from! { i16 => PositiveI16, u16 }
impl_negative! { #[repr(align(2))] NegativeI16, PositiveI16, i16, u16 }
impl_from_get! { NegativeI16 => NegativeI32, NegativeI64, NegativeIsize }
impl_primitive_from! { NegativeI16 => i16, i32, i64, i128, isize }
impl_primitive_try_from! { NegativeI16 => i8 }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI16, u16, i16 }

impl_positive! { #[repr(align(4))] PositiveI32, NegativeI32, i32, u32 }
impl_from! { u8, u16 => PositiveI32 }
impl_from_get! { PositiveI32 => PositiveI64 }
impl_primitive_from! { PositiveI32 => u32, u64, u128, i32, i64, i128 }
impl_primitive_try_from! { PositiveI32 => u8, u16, usize, i8, i16, isize }
impl_positive_try_from! { u32, u64, u128, usize => PositiveI32, i32 }
impl_positive_try_from! { i8, i16, i64, i128, isize => PositiveI32, u32, i32 }
impl_positive_try_from! { i32 => PositiveI32, u32 }
impl_negative! { #[repr(align(4))] NegativeI32, PositiveI32, i32, u32 }
impl_from_get! { NegativeI32 => NegativeI64 }
impl_primitive_from! { NegativeI32 => i32, i64, i128 }
impl_primitive_try_from! { NegativeI32 => i8, i16, isize }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI32, u32, i32 }

impl_positive! { #[repr(align(8))] PositiveI64, NegativeI64, i64, u64 }
impl_from! { u8, u16, u32 => PositiveI64 }
impl_primitive_from! { PositiveI64 => u64, u128, i64, i128 }
impl_primitive_try_from! { PositiveI64 => u8, u16, u32, usize, i8, i16, i32, isize }
impl_positive_try_from! { u64, u128, usize => PositiveI64, i64 }
impl_positive_try_from! { i8, i16, i32, i128, isize => PositiveI64, u64, i64 }
impl_positive_try_from! { i64 => PositiveI64, u64 }
impl_negative! { #[repr(align(8))] NegativeI64, PositiveI64, i64, u64 }
impl_primitive_from! { NegativeI64 => i64, i128 }
impl_primitive_try_from! { NegativeI64 => i8, i16, i32, isize }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI64, u64, i64 }

#[cfg(not(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64",
)))]
compile_error!("unsupported pointer width");

impl_positive! {
    #[cfg_attr(target_pointer_width = "16", repr(align(2)))]
    #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
    #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
    PositiveIsize, NegativeIsize, isize, usize
}
impl_from! { u8 => PositiveIsize }
impl_try_from! { PositiveIsize => PositiveI32, PositiveI64 }
impl_primitive_from! { PositiveIsize => usize, isize }
impl_primitive_try_from! { PositiveIsize => u8, u16, u32, u64, u128, i8, i16, i32, i64, i128 }
impl_positive_try_from! { u16, u32, u64, u128, usize => PositiveIsize, isize }
impl_positive_try_from! { i8, i16, i32, i64, i128 => PositiveIsize, usize, isize }
impl_positive_try_from! { isize => PositiveIsize, usize }
impl_negative! {
    #[cfg_attr(target_pointer_width = "16", repr(align(2)))]
    #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
    #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
    NegativeIsize, PositiveIsize, isize, usize
}
impl_try_from! { NegativeIsize => NegativeI32, NegativeI64 }
impl_primitive_from! { NegativeIsize => isize }
impl_primitive_try_from! { NegativeIsize => i8, i16, i32, i64, i128 }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeIsize, usize, isize }

#[derive(Copy, Clone)]
#[repr(u8)]
enum PositiveHighByte {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
    _4 = 4,
    _5 = 5,
    _6 = 6,
    _7 = 7,
    _8 = 8,
    _9 = 9,
    _10 = 10,
    _11 = 11,
    _12 = 12,
    _13 = 13,
    _14 = 14,
    _15 = 15,
    _16 = 16,
    _17 = 17,
    _18 = 18,
    _19 = 19,
    _20 = 20,
    _21 = 21,
    _22 = 22,
    _23 = 23,
    _24 = 24,
    _25 = 25,
    _26 = 26,
    _27 = 27,
    _28 = 28,
    _29 = 29,
    _30 = 30,
    _31 = 31,
    _32 = 32,
    _33 = 33,
    _34 = 34,
    _35 = 35,
    _36 = 36,
    _37 = 37,
    _38 = 38,
    _39 = 39,
    _40 = 40,
    _41 = 41,
    _42 = 42,
    _43 = 43,
    _44 = 44,
    _45 = 45,
    _46 = 46,
    _47 = 47,
    _48 = 48,
    _49 = 49,
    _50 = 50,
    _51 = 51,
    _52 = 52,
    _53 = 53,
    _54 = 54,
    _55 = 55,
    _56 = 56,
    _57 = 57,
    _58 = 58,
    _59 = 59,
    _60 = 60,
    _61 = 61,
    _62 = 62,
    _63 = 63,
    _64 = 64,
    _65 = 65,
    _66 = 66,
    _67 = 67,
    _68 = 68,
    _69 = 69,
    _70 = 70,
    _71 = 71,
    _72 = 72,
    _73 = 73,
    _74 = 74,
    _75 = 75,
    _76 = 76,
    _77 = 77,
    _78 = 78,
    _79 = 79,
    _80 = 80,
    _81 = 81,
    _82 = 82,
    _83 = 83,
    _84 = 84,
    _85 = 85,
    _86 = 86,
    _87 = 87,
    _88 = 88,
    _89 = 89,
    _90 = 90,
    _91 = 91,
    _92 = 92,
    _93 = 93,
    _94 = 94,
    _95 = 95,
    _96 = 96,
    _97 = 97,
    _98 = 98,
    _99 = 99,
    _100 = 100,
    _101 = 101,
    _102 = 102,
    _103 = 103,
    _104 = 104,
    _105 = 105,
    _106 = 106,
    _107 = 107,
    _108 = 108,
    _109 = 109,
    _110 = 110,
    _111 = 111,
    _112 = 112,
    _113 = 113,
    _114 = 114,
    _115 = 115,
    _116 = 116,
    _117 = 117,
    _118 = 118,
    _119 = 119,
    _120 = 120,
    _121 = 121,
    _122 = 122,
    _123 = 123,
    _124 = 124,
    _125 = 125,
    _126 = 126,
    _127 = 127,
}

#[derive(Copy, Clone)]
#[repr(u8)]
enum NegativeHighByte {
    _128 = 128,
    _129 = 129,
    _130 = 130,
    _131 = 131,
    _132 = 132,
    _133 = 133,
    _134 = 134,
    _135 = 135,
    _136 = 136,
    _137 = 137,
    _138 = 138,
    _139 = 139,
    _140 = 140,
    _141 = 141,
    _142 = 142,
    _143 = 143,
    _144 = 144,
    _145 = 145,
    _146 = 146,
    _147 = 147,
    _148 = 148,
    _149 = 149,
    _150 = 150,
    _151 = 151,
    _152 = 152,
    _153 = 153,
    _154 = 154,
    _155 = 155,
    _156 = 156,
    _157 = 157,
    _158 = 158,
    _159 = 159,
    _160 = 160,
    _161 = 161,
    _162 = 162,
    _163 = 163,
    _164 = 164,
    _165 = 165,
    _166 = 166,
    _167 = 167,
    _168 = 168,
    _169 = 169,
    _170 = 170,
    _171 = 171,
    _172 = 172,
    _173 = 173,
    _174 = 174,
    _175 = 175,
    _176 = 176,
    _177 = 177,
    _178 = 178,
    _179 = 179,
    _180 = 180,
    _181 = 181,
    _182 = 182,
    _183 = 183,
    _184 = 184,
    _185 = 185,
    _186 = 186,
    _187 = 187,
    _188 = 188,
    _189 = 189,
    _190 = 190,
    _191 = 191,
    _192 = 192,
    _193 = 193,
    _194 = 194,
    _195 = 195,
    _196 = 196,
    _197 = 197,
    _198 = 198,
    _199 = 199,
    _200 = 200,
    _201 = 201,
    _202 = 202,
    _203 = 203,
    _204 = 204,
    _205 = 205,
    _206 = 206,
    _207 = 207,
    _208 = 208,
    _209 = 209,
    _210 = 210,
    _211 = 211,
    _212 = 212,
    _213 = 213,
    _214 = 214,
    _215 = 215,
    _216 = 216,
    _217 = 217,
    _218 = 218,
    _219 = 219,
    _220 = 220,
    _221 = 221,
    _222 = 222,
    _223 = 223,
    _224 = 224,
    _225 = 225,
    _226 = 226,
    _227 = 227,
    _228 = 228,
    _229 = 229,
    _230 = 230,
    _231 = 231,
    _232 = 232,
    _233 = 233,
    _234 = 234,
    _235 = 235,
    _236 = 236,
    _237 = 237,
    _238 = 238,
    _239 = 239,
    _240 = 240,
    _241 = 241,
    _242 = 242,
    _243 = 243,
    _244 = 244,
    _245 = 245,
    _246 = 246,
    _247 = 247,
    _248 = 248,
    _249 = 249,
    _250 = 250,
    _251 = 251,
    _252 = 252,
    _253 = 253,
    _254 = 254,
    _255 = 255,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign, Not, Rem,
        RemAssign,
    };
    use proptest::prelude::*;

    macro_rules! test_unary {
        ($ty:ident, $base:ident ($range:expr) :: $($method:ident),+ $(,)?) => {
            proptest! {
                $(#[test] fn $method(n in $range) {
                    assert_eq!($ty::new(n).map(|n| n.$method()), Some((n as $base).$method()));
                })+
            }
        };
    }

    macro_rules! test_unary_op {
        ($ty:ident, $base:ident ($range:expr) :: $($method:ident),+ $(,)?) => {
            proptest! {
                $(#[test] fn $method(n in $range) {
                    assert_eq!($ty::new(n).map(|n| n.$method().get()), Some((n as $base).$method()));
                })+
            }
        };
    }

    macro_rules! test_unary_checked {
        ($ty:ident, $base:ident ($range:expr) :: $($method:ident),+ $(,)?) => {
            proptest! {
                $(#[test] fn $method(n in $range) {
                    assert_eq!($ty::new(n).and_then(|n| n.$method()), (n as $base).$method());
                })+
            }
        };
    }

    macro_rules! test_binary {
        ($ty:ident, $base:ident ($range1:expr, $range2:expr) :: $($method:ident),+ $(,)?) => {
            proptest! {
                $(#[test] fn $method(a in $range1, b in $range2) {
                    assert_eq!(
                        $ty::new(a).zip($ty::new(b)).map(|(a, b)| a.$method(b)).map(|a| a.get()),
                        Some((a as $base).$method(b as $base)),
                    );
                })+
            }
        };
    }

    macro_rules! test_binary_checked {
        ($ty:ident, $base:ident ($range1:expr, $range2:expr) :: $($method:ident),+ $(,)?) => {
            proptest! {
                $(#[test] fn $method(a in $range1, b in $range2) {
                    assert_eq!(
                        $ty::new(a).zip($ty::new(b)).and_then(|(a, b)| a.$method(b)).map(|a| a.get()),
                        (a as $base).$method(b as $base),
                    );
                })+
            }
        };
    }

    macro_rules! test_assign {
        ($ty:ident, $base:ident ($range1:expr, $range2:expr) :: $($method:ident),+ $(,)?) => {
            proptest! {
                $(#[test] fn $method(a in $range1, b in $range2) {
                    let mut a1 = $ty::new(a);
                    let mut a2 = a;
                    if let Some((a, b)) = a1.as_mut().zip($ty::new(b)) {
                        a.$method(b);
                    }
                    a2.$method(b);
                    assert_eq!(a1.map(|a| a.get()), Some(a2));
                })+
            }
        };
    }

    macro_rules! test_type {
        ($base:ident, $uns:ident, $pos:ident, $neg:ident) => {
            mod $base {
                use super::*;
                mod positive {
                    use super::*;
                    proptest! {
                        #[test]
                        fn valid(n in 0..=$base::MAX) {
                            assert_eq!($pos::new(n).map(|n| n.get()), Some(n));
                        }
                        #[test]
                        fn invalid(n in $base::MIN..-1) {
                            assert_eq!($pos::new(n).map(|n| n.get()), None);
                        }
                        #[test]
                        fn checked_neg(n in 0..=$base::MAX) {
                            assert_eq!(
                                $pos::new(n).and_then(|n| n.checked_neg()),
                                n.checked_neg().and_then($neg::new),
                            );
                        }
                        #[test]
                        fn checked_sub(a in 0..=$base::MAX, b in 0..=$base::MAX) {
                            assert_eq!(
                                $pos::new(a).zip($pos::new(b)).and_then(|(a, b)| a.checked_sub(b)),
                                a.checked_sub(b).and_then($pos::new),
                            );
                        }
                        #[test]
                        fn checked_div_unsigned(a in 0..=$base::MAX, b in 0..=$uns::MAX) {
                            assert_eq!(
                                $pos::new(a).and_then(|a| a.checked_div_unsigned(b)),
                                (a as $uns).checked_div(b).and_then(|n| $pos::try_from(n).ok()),
                            );
                        }
                        #[test]
                        fn checked_rem_unsigned(a in 0..=$base::MAX, b in 0..=$uns::MAX) {
                            assert_eq!(
                                $pos::new(a).and_then(|a| a.checked_rem_unsigned(b)),
                                (a as $uns).checked_rem(b).and_then(|n| $pos::try_from(n).ok()),
                            );
                        }
                        #[test]
                        fn checked_pow(a in 0..=$base::MAX, b in 0..u32::MAX) {
                            assert_eq!(
                                $pos::new(a).and_then(|a| a.checked_pow(b)).map(|n| n.get()),
                                a.checked_pow(b),
                            );
                        }
                        #[test]
                        fn checked_next_power_of_two(n in 0..=$base::MAX) {
                            assert_eq!(
                                $pos::new(n).and_then(|n| n.checked_next_power_of_two()),
                                (n as $uns).checked_next_power_of_two().and_then(|n| $pos::try_from(n).ok()),
                            );
                        }
                        #[test]
                        fn saturating_sub(a in 0..=$base::MAX, b in 0..=$base::MAX) {
                            assert_eq!(
                                $pos::new(a).zip($pos::new(b)).map(|(a, b)| a.saturating_sub(b)).map(|a| a.get()),
                                Some(a.saturating_sub(b).max(0)),
                            );
                        }
                        #[test]
                        fn saturating_pow(a in 0..=$base::MAX, b in 0..u32::MAX) {
                            assert_eq!(
                                $pos::new(a).map(|a| a.saturating_pow(b)).map(|a| a.get()),
                                Some((a as $base).saturating_pow(b)),
                            );
                        }
                    }
                    test_unary_op! { $pos, $base (0..=$base::MAX) :: not }
                    test_binary! { $pos, $base (0..=$base::MAX, 1..=$base::MAX) :: div, rem }
                    test_unary! { $pos, $base (0..=$base::MAX)
                    :: count_zeros, count_ones, leading_zeros, trailing_zeros }
                    test_unary! { $pos, $uns (0..=$base::MAX) :: is_power_of_two }
                    test_unary! { $pos, $base (1..=$base::MAX) :: ilog2, ilog10 }
                    test_unary_checked! { $pos, $base (1..=$base::MAX) :: checked_ilog2, checked_ilog10 }
                    test_binary_checked! { $pos, $base (0..=$base::MAX, 0..=$base::MAX)
                    :: checked_add, checked_mul, checked_div, checked_rem }
                    test_binary! { $pos, $base (0..=$base::MAX, 0..=$base::MAX)
                    :: saturating_add, saturating_mul, bitor, bitand, bitxor }
                    test_assign! { $pos, $base (0..=$base::MAX, 1..=$base::MAX) :: div_assign, rem_assign }
                    test_assign! { $pos, $base (0..=$base::MAX, 0..=$base::MAX)
                    :: bitor_assign, bitand_assign, bitxor_assign }
                }
                mod negative {
                    use super::*;
                    proptest! {
                        #[test]
                        fn valid(n in $base::MIN..0) {
                            assert_eq!($neg::new(n).map(|n| n.get()), Some(n));
                        }
                        #[test]
                        fn invalid(n in 0..=$base::MAX) {
                            assert_eq!($neg::new(n).map(|n| n.get()), None);
                        }
                        #[test]
                        fn checked_abs(n in $base::MIN..0) {
                            assert_eq!(
                                $neg::new(n).and_then(|n| n.checked_abs()).map(|n| n.get()),
                                n.checked_abs(),
                            );
                        }
                        #[test]
                        fn checked_neg(n in $base::MIN..0) {
                            assert_eq!(
                                $neg::new(n).and_then(|n| n.checked_neg()).map(|n| n.get()),
                                n.checked_neg(),
                            );
                        }
                        #[test]
                        fn checked_sub(a in $base::MIN..0, b in $base::MIN..0) {
                            assert_eq!(
                                $neg::new(a).zip($neg::new(b)).and_then(|(a, b)| a.checked_sub(b)),
                                a.checked_sub(b).and_then($neg::new),
                            );
                        }
                        #[test]
                        fn checked_mul_positive(a in $base::MIN..0, b in 0..=$base::MAX) {
                            assert_eq!(
                                $neg::new(a)
                                    .zip($pos::new(b))
                                    .and_then(|(a, b)| a.checked_mul_positive(b)),
                                a.checked_mul(b).and_then($neg::new),
                            );
                        }
                        #[test]
                        fn checked_rem_euclid(a in $base::MIN..0, b in $base::MIN..=$base::MAX) {
                            assert_eq!(
                                $neg::new(a).and_then(|a| a.checked_rem_euclid(b)).map(|n| n.get()),
                                a.checked_rem_euclid(b),
                            );
                        }
                        #[test]
                        fn saturating_abs(n in $base::MIN..0) {
                            assert_eq!(
                                $neg::new(n).map(|n| n.saturating_abs().get()),
                                Some(n.saturating_abs()),
                            );
                        }
                        #[test]
                        fn saturating_neg(n in $base::MIN..0) {
                            assert_eq!(
                                $neg::new(n).map(|n| n.saturating_neg().get()),
                                Some(n.saturating_neg()),
                            );
                        }
                        #[test]
                        fn saturating_sub(a in $base::MIN..0, b in $base::MIN..0) {
                            assert_eq!(
                                $neg::new(a)
                                    .zip($neg::new(b))
                                    .map(|(a, b)| a.saturating_sub(b).get()),
                                Some(a.saturating_sub(b).min(-1)),
                            );
                        }
                        #[test]
                        fn saturating_mul_positive(a in $base::MIN..0, b in 0..=$base::MAX) {
                            assert_eq!(
                                $neg::new(a)
                                    .zip($pos::new(b))
                                    .map(|(a, b)| a.saturating_mul_positive(b).get()),
                                Some(a.saturating_mul(b).min(-1)),
                            );
                        }
                        #[test]
                        fn bitxor_assign(a in $base::MIN..0, b in 0..=$base::MAX) {
                            let mut a1 = $neg::new(a);
                            let mut a2 = a;
                            if let Some((a, b)) = a1.as_mut().zip($pos::new(b)) {
                                a.bitxor_assign(b);
                            }
                            a2.bitxor_assign(b);
                            assert_eq!(a1.map(|a| a.get()), Some(a2));
                        }
                    }
                    test_unary_op! { $neg, $base ($base::MIN..0) :: not }
                    test_unary! { $neg, $base ($base::MIN..0)
                    :: count_zeros, count_ones, leading_zeros, trailing_zeros }
                    test_binary_checked! { $neg, $base ($base::MIN..0, $base::MIN..0)
                    :: checked_add, checked_mul, checked_div, checked_div_euclid }
                    test_binary! { $neg, $base ($base::MIN..0, $base::MIN..0)
                    :: saturating_add, saturating_mul, bitor, bitand, bitxor }
                    test_assign! { $neg, $base ($base::MIN..0, $base::MIN..0)
                    :: bitor_assign, bitand_assign }
                }
            }
        };
    }
    test_type! { i8, u8, PositiveI8, NegativeI8 }
    test_type! { i16, u16, PositiveI16, NegativeI16 }
    test_type! { i32, u32, PositiveI32, NegativeI32 }
    test_type! { i64, u64, PositiveI64, NegativeI64 }
    test_type! { isize, usize, PositiveIsize, NegativeIsize }
}
