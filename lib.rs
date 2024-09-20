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

#![no_std]

macro_rules! impl_positive {
    ($(#[$attr:meta])* $ty:ident, $base:ty, $uns:ty) => {
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
            pub const BITS: u32 = <$base>::BITS;
            pub const MIN: Self = unsafe { $ty::new_unchecked(0) };
            pub const MAX: Self = unsafe { $ty::new_unchecked(<$base>::MAX) };
            pub const fn new(value: $base) -> Option<Self> {
                if value < 0 {
                    return None;
                }
                unsafe { Some(Self::new_unchecked(value)) }
            }
            #[inline]
            pub const unsafe fn new_unchecked(value: $base) -> Self {
                debug_assert!(value >= 0);
                core::mem::transmute(value)
            }
            #[inline]
            pub const fn get(self) -> $base {
                unsafe { core::mem::transmute(self) }
            }
            #[inline]
            pub const fn count_zeros(self) -> u32 {
                self.get().count_zeros()
            }
            #[inline]
            pub const fn count_ones(self) -> u32 {
                self.get().count_ones()
            }
            #[inline]
            pub const fn leading_zeros(self) -> u32 {
                self.get().leading_zeros()
            }
            #[inline]
            pub const fn trailing_zeros(self) -> u32 {
                self.get().trailing_zeros()
            }
            #[inline]
            pub const fn is_power_of_two(self) -> bool {
                (self.get() as $uns).is_power_of_two()
            }
            #[inline]
            pub const fn ilog2(self) -> u32 {
                self.get().ilog2()
            }
            #[inline]
            pub const fn ilog10(self) -> u32 {
                self.get().ilog10()
            }
            #[inline]
            pub const fn checked_add(self, other: Self) -> Option<Self> {
                match self.get().checked_add(other.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_mul(self, other: Self) -> Option<Self> {
                match self.get().checked_mul(other.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_pow(self, other: u32) -> Option<Self> {
                match self.get().checked_pow(other) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_next_power_of_two(self) -> Option<Self> {
                match (self.get() as $uns).checked_next_power_of_two() {
                    Some(n) => Self::new(n as $base),
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_ilog2(self) -> Option<u32> {
                self.get().checked_ilog2()
            }
            #[inline]
            pub const fn checked_ilog10(self) -> Option<u32> {
                self.get().checked_ilog10()
            }
            #[inline]
            pub const fn saturating_add(self, other: Self) -> Self {
                let n = self.get().saturating_add(other.get());
                unsafe { Self::new_unchecked(n) }
            }
            #[inline]
            pub const fn saturating_mul(self, other: Self) -> Self {
                let n = self.get().saturating_mul(other.get());
                unsafe { Self::new_unchecked(n) }
            }
            #[inline]
            pub const fn saturating_pow(self, other: u32) -> Self {
                let n = self.get().saturating_pow(other);
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
            fn eq(&self, other: &Self) -> bool {
                self.get().eq(&other.get())
            }
        }

        impl PartialOrd for $ty {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.get().partial_cmp(&other.get())
            }
        }

        impl Ord for $ty {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.get().cmp(&other.get())
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
    ($(#[$attr:meta])* $ty:ident, $unty:ident, $base:ty, $uns:ty) => {
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
            pub const BITS: u32 = <$base>::BITS;
            pub const MIN: Self = unsafe { $ty::new_unchecked(<$base>::MIN) };
            pub const MAX: Self = unsafe { $ty::new_unchecked(-1) };
            pub const fn new(value: $base) -> Option<Self> {
                if value >= 0 {
                    return None;
                }
                unsafe { Some(Self::new_unchecked(value)) }
            }
            #[inline]
            pub const unsafe fn new_unchecked(value: $base) -> Self {
                debug_assert!(value < 0);
                core::mem::transmute(value)
            }
            #[inline]
            pub const fn get(self) -> $base {
                unsafe { core::mem::transmute(self) }
            }
            #[inline]
            pub const fn count_zeros(self) -> u32 {
                self.get().count_zeros()
            }
            #[inline]
            pub const fn count_ones(self) -> u32 {
                self.get().count_ones()
            }
            #[inline]
            pub const fn leading_zeros(self) -> u32 {
                self.get().leading_zeros()
            }
            #[inline]
            pub const fn trailing_zeros(self) -> u32 {
                self.get().trailing_zeros()
            }
            #[inline]
            pub const fn abs(self) -> $unty {
                let n = self.get().abs();
                unsafe { $unty::new_unchecked(n) }
            }
            #[inline]
            pub const fn unsigned_abs(self) -> $uns {
                self.get().unsigned_abs()
            }
            #[inline]
            pub const fn checked_abs(self) -> Option<$unty> {
                match self.get().checked_abs() {
                    Some(n) => unsafe { Some($unty::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_neg(self) -> Option<$unty> {
                match self.get().checked_neg() {
                    Some(n) => unsafe { Some($unty::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_sub(self, other: Self) -> Option<Self> {
                match self.get().checked_add(other.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn checked_mul(self, other: $unty) -> Option<Self> {
                match self.get().checked_mul(other.get()) {
                    Some(n) => unsafe { Some(Self::new_unchecked(n)) },
                    None => None,
                }
            }
            #[inline]
            pub const fn saturating_abs(self) -> $unty {
                let n = self.get().saturating_abs();
                unsafe { $unty::new_unchecked(n) }
            }
            #[inline]
            pub const fn saturating_neg(self) -> $unty {
                let n = self.get().saturating_neg();
                unsafe { $unty::new_unchecked(n) }
            }
            #[inline]
            pub const fn saturating_sub(self, other: Self) -> Self {
                let n = self.get().saturating_sub(other.get());
                unsafe { Self::new_unchecked(n) }
            }
            #[inline]
            pub const fn saturating_mul(self, other: $unty) -> Self {
                let n = self.get().saturating_mul(other.get());
                unsafe { Self::new_unchecked(n) }
            }
        }

        impl PartialEq for $ty {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                self.get().eq(&other.get())
            }
        }

        impl PartialOrd for $ty {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.get().partial_cmp(&other.get())
            }
        }

        impl Ord for $ty {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.get().cmp(&other.get())
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

        impl_fmt! { Display, Debug, Binary, Octal, LowerHex, UpperHex => $ty }
        impl_bit_op! { BitOr::bitor, BitOrAssign::bitor_assign for $ty }
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

impl_positive! { #[repr(align(1))] PositiveI8, i8, u8 }
impl_positive_try_from! { u8, u16, u32, u64, u128, usize => PositiveI8, i8 }
impl_positive_try_from! { i16, i32, i64, i128, isize => PositiveI8, u8, i8 }
impl_positive_try_from! { i8 => PositiveI8, u8 }
impl_negative! { #[repr(align(1))] NegativeI8, PositiveI8, i8, u8 }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI8, u8, i8 }

impl_positive! { #[repr(align(2))] PositiveI16, i16, u16 }
impl_from! { u8 => PositiveI16 }
impl_positive_try_from! { u16, u32, u64, u128, usize => PositiveI16, i16 }
impl_positive_try_from! { i8, i32, i64, i128, isize => PositiveI16, u16, i16 }
impl_positive_try_from! { i16 => PositiveI16, u16 }
impl_negative! { #[repr(align(2))] NegativeI16, PositiveI16, i16, u16 }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI16, u16, i16 }

impl_positive! { #[repr(align(4))] PositiveI32, i32, u32 }
impl_from! { u8, u16 => PositiveI32 }
impl_positive_try_from! { u32, u64, u128, usize => PositiveI32, i32 }
impl_positive_try_from! { i8, i16, i64, i128, isize => PositiveI32, u32, i32 }
impl_positive_try_from! { i32 => PositiveI32, u32 }
impl_negative! { #[repr(align(4))] NegativeI32, PositiveI32, i32, u32 }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI32, u32, i32 }

impl_positive! { #[repr(align(8))] PositiveI64, i64, u64 }
impl_from! { u8, u16, u32 => PositiveI64 }
impl_positive_try_from! { u64, u128, usize => PositiveI64, i64 }
impl_positive_try_from! { i8, i16, i32, i128, isize => PositiveI64, u64, i64 }
impl_positive_try_from! { i64 => PositiveI64, u64 }
impl_negative! { #[repr(align(8))] NegativeI64, PositiveI64, i64, u64 }
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeI64, u64, i64 }

impl_positive! {
    #[cfg_attr(target_pointer_width = "16", repr(align(2)))]
    #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
    #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
    PositiveIsize, isize, usize
}
impl_from! { u8, u16 => PositiveIsize }
impl_positive_try_from! { u32, u64, u128, usize => PositiveIsize, isize }
impl_positive_try_from! { i8, i16, i32, i64, i128 => PositiveIsize, usize, isize }
impl_positive_try_from! { isize => PositiveIsize, usize }
impl_negative! {
    #[cfg_attr(target_pointer_width = "16", repr(align(2)))]
    #[cfg_attr(target_pointer_width = "32", repr(align(4)))]
    #[cfg_attr(target_pointer_width = "64", repr(align(8)))]
    NegativeIsize, PositiveIsize, isize, usize
}
impl_negative_try_from! { i8, i16, i32, i64, i128, isize => NegativeIsize, usize, isize }

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    macro_rules! test_type {
        ($base:ident, $pos:ident, $neg:ident) => {
            mod $base {
                use super::*;
                proptest! {
                    #[test]
                    fn positive_valid(n in 0..$base::MAX) {
                        assert_eq!($pos::new(n).map(|n| n.get()), Some(n));
                    }
                    #[test]
                    fn positive_invalid(n in $base::MIN..-1) {
                        assert_eq!($pos::new(n).map(|n| n.get()), None);
                    }
                    #[test]
                    fn negative_valid(n in $base::MIN..-1) {
                        assert_eq!($neg::new(n).map(|n| n.get()), Some(n));
                    }
                    #[test]
                    fn negative_invalid(n in 0..$base::MAX) {
                        assert_eq!($neg::new(n).map(|n| n.get()), None);
                    }
                }
            }
        };
    }
    test_type! { i8, PositiveI8, NegativeI8 }
    test_type! { i16, PositiveI16, NegativeI16 }
    test_type! { i32, PositiveI32, NegativeI32 }
    test_type! { i64, PositiveI64, NegativeI64 }
    test_type! { isize, PositiveIsize, NegativeIsize }
}
