use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, ShlAssign, Sub, SubAssign};

const BIAS: i32 = 16383;
const SPECIAL: i32 = 0x7FFF;
const INTEGER_BIT: u64 = 1 << 63;

#[derive(Clone, Copy)]
pub struct F80 {
    sign: bool,
    exponent: u16,
    mantissa: u64,
}

impl F80 {
    pub const fn zero(sign: bool) -> Self {
        Self { sign, exponent: 0, mantissa: 0 }
    }

    pub const fn infinity(sign: bool) -> Self {
        Self { sign, exponent: SPECIAL as u16, mantissa: INTEGER_BIT }
    }

    pub const fn nan() -> Self {
        Self { sign: false, exponent: SPECIAL as u16, mantissa: INTEGER_BIT | 1 << 62 }
    }

    pub fn is_nan(self) -> bool {
        self.exponent as i32 == SPECIAL && self.mantissa != INTEGER_BIT
    }

    pub fn is_infinite(self) -> bool {
        self.exponent as i32 == SPECIAL && self.mantissa == INTEGER_BIT
    }

    pub fn is_finite(self) -> bool {
        self.exponent as i32 != SPECIAL
    }

    pub fn is_zero(self) -> bool {
        self.exponent == 0 && self.mantissa == 0
    }

    pub fn is_negative(self) -> bool {
        self.sign && !self.is_zero() && !self.is_nan()
    }

    pub fn to_bits(self) -> (u16, u64) {
        ((self.sign as u16) << 15 | self.exponent, self.mantissa)
    }

    fn parts(self) -> (u64, i32) {
        let unbiased = match self.exponent {
            0 => 1 - BIAS - 63,
            biased => biased as i32 - BIAS - 63,
        };
        (self.mantissa, unbiased)
    }

    fn normalized(self) -> (u64, i32) {
        let (mantissa, exp) = self.parts();
        let shift = mantissa.leading_zeros();
        (mantissa << shift, exp - shift as i32)
    }

    fn magnitude(self) -> (u16, u64) {
        (self.exponent, self.mantissa)
    }

    fn truncated(self) -> Option<u128> {
        if !self.is_finite() {
            return None;
        }
        let (mantissa, exp) = self.parts();
        if exp >= 64 {
            return None;
        }
        if exp >= 0 {
            return Some(u128::from(mantissa) << exp);
        }
        if -exp >= 128 {
            return Some(0);
        }
        Some(u128::from(mantissa) >> (-exp))
    }
}

fn scale(mut value: f64, mut exp: i32) -> f64 {
    const STEP: i32 = 1000;
    let up = 2f64.powi(STEP);
    if exp > STEP {
        return f64::INFINITY;
    }
    while exp < -STEP {
        value /= up;
        exp += STEP;
        if value == 0.0 {
            return value;
        }
    }
    value * 2f64.powi(exp)
}

fn round_pack(sign: bool, mut mantissa: u128, mut exp: i32, mut sticky: bool) -> F80 {
    if mantissa == 0 {
        return F80::zero(sign);
    }
    let mut guard = false;
    let width = 128 - mantissa.leading_zeros() as i32;
    match width.cmp(&64) {
        Ordering::Greater => {
            let drop = (width - 64) as u32;
            let lost = mantissa & ((1 << drop) - 1);
            guard = lost >> (drop - 1) != 0;
            sticky |= lost & ((1 << (drop - 1)) - 1) != 0;
            mantissa >>= drop;
            exp += drop as i32;
        }
        Ordering::Less => {
            let gain = (64 - width) as u32;
            mantissa <<= gain;
            exp -= gain as i32;
        }
        Ordering::Equal => (),
    }

    let mut biased = exp + BIAS + 63;
    if biased <= 0 {
        let shift = 1 - biased;
        if shift > 66 {
            return F80::zero(sign);
        }
        for _ in 0..shift {
            sticky |= guard;
            guard = mantissa & 1 != 0;
            mantissa >>= 1;
        }
        biased = 0;
    }

    if guard && (sticky || mantissa & 1 != 0) {
        mantissa += 1;
        if mantissa >> 64 != 0 {
            mantissa >>= 1;
            biased += 1;
        }
    }
    if biased == 0 && mantissa >> 63 != 0 {
        biased = 1;
    }
    if biased >= SPECIAL {
        return F80::infinity(sign);
    }
    F80 { sign, exponent: biased as u16, mantissa: mantissa as u64 }
}

impl From<&str> for F80 {
    fn from(s: &str) -> Self {
        let bytes = s.as_bytes();
        let mut digits: Vec<u8> = Vec::new();
        let mut exponent: i64 = 0;
        let mut fraction = false;
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                c if c.is_ascii_digit() => {
                    digits.push(c - b'0');
                    if fraction {
                        exponent -= 1;
                    }
                }
                b'.' => fraction = true,
                _ => break,
            }
            i += 1;
        }
        if i < bytes.len() && (bytes[i] | 0x20) == b'e' {
            i += 1;
            let negative = i < bytes.len() && bytes[i] == b'-';
            if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
                i += 1;
            }
            let mut written: i64 = 0;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                written = (written * 10 + i64::from(bytes[i] - b'0')).min(1 << 40);
                i += 1;
            }
            exponent += if negative { -written } else { written };
        }

        let Some(first) = digits.iter().position(|&d| d != 0) else {
            return Self::zero(false);
        };
        let digits = &digits[first..];

        let magnitude = exponent + digits.len() as i64;
        if magnitude > 4933 {
            return Self::infinity(false);
        }
        if magnitude < -4960 {
            return Self::zero(false);
        }

        let mut numerator = Big::default();
        for &digit in digits {
            numerator *= 10;
            numerator += u32::from(digit);
        }
        let mut denominator = Big::from(1u32);
        match exponent.cmp(&0) {
            Ordering::Greater => numerator.mul_pow10(exponent as u64),
            Ordering::Less => denominator.mul_pow10((-exponent) as u64),
            Ordering::Equal => (),
        }

        let shift = 66 + i64::from(denominator.bit_len()) - i64::from(numerator.bit_len());
        match shift.cmp(&0) {
            Ordering::Greater => numerator <<= shift as u32,
            Ordering::Less => denominator <<= (-shift) as u32,
            Ordering::Equal => (),
        }
        let (quotient, remainder) = Big::div_rem(&numerator, &denominator);
        round_pack(false, u128::from(&quotient), -(shift as i32), !remainder.is_zero())
    }
}

impl From<u64> for F80 {
    fn from(value: u64) -> Self {
        round_pack(false, u128::from(value), 0, false)
    }
}

impl From<i64> for F80 {
    fn from(value: i64) -> Self {
        let magnitude = Self::from(value.unsigned_abs());
        if value < 0 { -magnitude } else { magnitude }
    }
}

impl From<f64> for F80 {
    fn from(value: f64) -> Self {
        if value.is_nan() {
            return Self::nan();
        }
        if value.is_infinite() {
            return Self::infinity(value.is_sign_negative());
        }
        let bits = value.to_bits();
        let sign = bits >> 63 != 0;
        if value == 0.0 {
            return Self::zero(sign);
        }
        let biased = (bits >> 52 & 0x7FF) as i32;
        let fraction = bits & ((1 << 52) - 1);
        let (mantissa, exp) = match biased {
            0 => (fraction, 1 - 1023 - 52),
            _ => (fraction | 1 << 52, biased - 1023 - 52),
        };
        round_pack(sign, u128::from(mantissa), exp, false)
    }
}

impl From<F80> for f64 {
    fn from(value: F80) -> Self {
        if value.is_nan() {
            return f64::NAN;
        }
        if value.is_infinite() {
            return if value.sign { f64::NEG_INFINITY } else { f64::INFINITY };
        }
        if value.is_zero() {
            return if value.sign { -0.0 } else { 0.0 };
        }
        let (mantissa, exp) = value.parts();
        let magnitude = scale(mantissa as f64, exp);
        if value.sign { -magnitude } else { magnitude }
    }
}

impl From<F80> for i64 {
    fn from(value: F80) -> Self {
        if value.is_nan() {
            return 0;
        }
        let Some(magnitude) = value.truncated() else {
            return if value.sign { i64::MIN } else { i64::MAX };
        };
        if value.sign {
            match magnitude > 1 << 63 {
                true => i64::MIN,
                false => (magnitude as i64).wrapping_neg(),
            }
        } else {
            match magnitude > i64::MAX as u128 {
                true => i64::MAX,
                false => magnitude as i64,
            }
        }
    }
}

impl From<F80> for u64 {
    fn from(value: F80) -> Self {
        if value.is_nan() || value.sign {
            return 0;
        }
        match value.truncated() {
            None => u64::MAX,
            Some(magnitude) if magnitude > u64::MAX as u128 => u64::MAX,
            Some(magnitude) => magnitude as u64,
        }
    }
}

impl Neg for F80 {
    type Output = Self;

    fn neg(self) -> Self {
        Self { sign: !self.sign, ..self }
    }
}

impl Add for F80 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if self.is_nan() || rhs.is_nan() {
            return Self::nan();
        }
        if self.is_infinite() || rhs.is_infinite() {
            if self.is_infinite() && rhs.is_infinite() && self.sign != rhs.sign {
                return Self::nan();
            }
            return if self.is_infinite() { self } else { rhs };
        }
        if self.is_zero() && rhs.is_zero() {
            return Self::zero(self.sign && rhs.sign);
        }
        if self.is_zero() {
            return rhs;
        }
        if rhs.is_zero() {
            return self;
        }

        let (big, small) = match rhs.magnitude() > self.magnitude() {
            true => (rhs, self),
            false => (self, rhs),
        };
        let (big_mantissa, big_exp) = big.normalized();
        let (small_mantissa, small_exp) = small.normalized();

        const ROOM: i32 = 63;
        let scaled = u128::from(big_mantissa) << ROOM;
        let align = ROOM - (big_exp - small_exp);
        let (addend, sticky) = if align >= 0 {
            (u128::from(small_mantissa) << align, false)
        } else if -align >= 128 {
            (0, true)
        } else {
            let drop = (-align) as u32;
            let lost = u128::from(small_mantissa) & ((1 << drop) - 1);
            (u128::from(small_mantissa) >> drop, lost != 0)
        };

        let exp = big_exp - ROOM;
        if big.sign == small.sign {
            return round_pack(big.sign, scaled + addend, exp, sticky);
        }
        let mut difference = scaled - addend;
        if difference == 0 {
            return Self::zero(false);
        }
        if sticky {
            difference -= 1;
        }
        round_pack(big.sign, difference, exp, sticky)
    }
}

impl Sub for F80 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl Mul for F80 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.is_nan() || rhs.is_nan() {
            return Self::nan();
        }
        let sign = self.sign ^ rhs.sign;
        if self.is_infinite() || rhs.is_infinite() {
            if self.is_zero() || rhs.is_zero() {
                return Self::nan();
            }
            return Self::infinity(sign);
        }
        if self.is_zero() || rhs.is_zero() {
            return Self::zero(sign);
        }
        let (lhs_mantissa, lhs_exp) = self.normalized();
        let (rhs_mantissa, rhs_exp) = rhs.normalized();
        let product = u128::from(lhs_mantissa) * u128::from(rhs_mantissa);
        round_pack(sign, product, lhs_exp + rhs_exp, false)
    }
}

impl Div for F80 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        if self.is_nan() || rhs.is_nan() {
            return Self::nan();
        }
        let sign = self.sign ^ rhs.sign;
        if self.is_infinite() {
            return if rhs.is_infinite() { Self::nan() } else { Self::infinity(sign) };
        }
        if rhs.is_infinite() {
            return Self::zero(sign);
        }
        if rhs.is_zero() {
            return if self.is_zero() { Self::nan() } else { Self::infinity(sign) };
        }
        if self.is_zero() {
            return Self::zero(sign);
        }
        let (lhs_mantissa, lhs_exp) = self.normalized();
        let (rhs_mantissa, rhs_exp) = rhs.normalized();
        let numerator = u128::from(lhs_mantissa) << 64;
        let denominator = u128::from(rhs_mantissa);
        let quotient = numerator / denominator;
        let remainder = numerator % denominator;
        let carry = u128::from(2 * remainder >= denominator);
        let doubled = quotient * 2 + carry;
        let left = 2 * remainder - carry * denominator;
        round_pack(sign, doubled, lhs_exp - rhs_exp - 65, left != 0)
    }
}

impl PartialEq for F80 {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for F80 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.is_nan() || other.is_nan() {
            return None;
        }
        if self.is_zero() && other.is_zero() {
            return Some(Ordering::Equal);
        }
        Some(match (self.sign, other.sign) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => self.magnitude().cmp(&other.magnitude()),
            (true, true) => other.magnitude().cmp(&self.magnitude()),
        })
    }
}

impl fmt::UpperHex for F80 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (high, low) = self.to_bits();
        write!(f, "0xK{high:04X}{low:016X}")
    }
}

impl fmt::Display for F80 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", f64::from(*self))
    }
}

impl fmt::Debug for F80 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", f64::from(*self))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Big(Vec<u32>);

impl Big {
    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    fn trim(&mut self) {
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    fn bit_len(&self) -> u32 {
        let top = self.0.last().expect("a non-zero magnitude");
        self.0.len() as u32 * 32 - top.leading_zeros()
    }

    fn bit(&self, index: u32) -> bool {
        self.0.get((index / 32) as usize).is_some_and(|limb| limb >> (index % 32) & 1 == 1)
    }

    fn set_bit(&mut self, index: u32) {
        let limb = (index / 32) as usize;
        if self.0.len() <= limb {
            self.0.resize(limb + 1, 0);
        }
        self.0[limb] |= 1 << (index % 32);
    }

    fn mul_pow10(&mut self, mut power: u64) {
        while power >= 9 {
            *self *= 1_000_000_000;
            power -= 9;
        }
        if power > 0 {
            *self *= 10u32.pow(power as u32);
        }
    }

    fn div_rem(numerator: &Self, denominator: &Self) -> (Self, Self) {
        let mut quotient = Self::default();
        let mut remainder = Self::default();
        for i in (0..numerator.bit_len()).rev() {
            remainder <<= 1;
            if numerator.bit(i) {
                remainder.set_bit(0);
            }
            if remainder >= *denominator {
                remainder -= denominator;
                quotient.set_bit(i);
            }
        }
        quotient.trim();
        remainder.trim();
        (quotient, remainder)
    }
}

impl From<u32> for Big {
    fn from(value: u32) -> Self {
        let mut big = Big(vec![value]);
        big.trim();
        big
    }
}

impl From<&Big> for u128 {
    fn from(value: &Big) -> Self {
        value.0.iter().take(4).enumerate().fold(0, |packed, (i, limb)| packed | u128::from(*limb) << (32 * i))
    }
}

impl Ord for Big {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.len().cmp(&other.0.len()).then_with(|| self.0.iter().rev().cmp(other.0.iter().rev()))
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl MulAssign<u32> for Big {
    fn mul_assign(&mut self, factor: u32) {
        let mut carry = 0u64;
        for limb in self.0.iter_mut() {
            let wide = u64::from(*limb) * u64::from(factor) + carry;
            *limb = wide as u32;
            carry = wide >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
        self.trim();
    }
}

impl AddAssign<u32> for Big {
    fn add_assign(&mut self, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in self.0.iter_mut() {
            if carry == 0 {
                return;
            }
            let wide = u64::from(*limb) + carry;
            *limb = wide as u32;
            carry = wide >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }
}

impl SubAssign<&Big> for Big {
    fn sub_assign(&mut self, other: &Big) {
        let mut borrow = 0i64;
        for i in 0..self.0.len() {
            let rhs = i64::from(*other.0.get(i).unwrap_or(&0));
            let wide = i64::from(self.0[i]) - rhs - borrow;
            let (value, next) = match wide < 0 {
                true => (wide + (1 << 32), 1),
                false => (wide, 0),
            };
            self.0[i] = value as u32;
            borrow = next;
        }
        self.trim();
    }
}

impl ShlAssign<u32> for Big {
    fn shl_assign(&mut self, count: u32) {
        if self.is_zero() || count == 0 {
            return;
        }
        let bits = count % 32;
        if bits > 0 {
            let mut carry = 0u32;
            for limb in self.0.iter_mut() {
                let wide = u64::from(*limb) << bits | u64::from(carry);
                *limb = wide as u32;
                carry = (wide >> 32) as u32;
            }
            if carry != 0 {
                self.0.push(carry);
            }
        }
        let limbs = (count / 32) as usize;
        if limbs > 0 {
            self.0.splice(0..0, vec![0u32; limbs]);
        }
    }
}
