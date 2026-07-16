use crate::{
    error::Fallible,
    traits::{NextFloat, samplers::sample_from_uniform_bytes},
};

#[cfg(test)]
mod test;

const NATIVE_UNIFORM_REFINE_BITS: u32 = 4;

// Fixed-width pure-RDP profile for DP-SGD on binary32 GPUs.
//
// The exact-normal sampler uses a 112-bit partial-uniform comparison cap,
// K <= 2^20 - 1, and at most 32 rounds per Bernoulli factory.  Structural
// shell retry and sampler-resource retry are the only conditioned events.
// After acceptance, a scale-aligned mantissa split and a 64-bit completion
// bin realize the exact clipped-and-rounded binary32 law.  The aligned affine
// coordinate is bounded to 160 signed bits.  Unsupported public parameter
// profiles are rejected before private sampling begins.
const NATIVE_SAMPLER_UNIFORM_MAX_BITS: u32 = 112;
const NATIVE_COMPLETION_BITS: u32 = 64;
const NATIVE_SAMPLER_K_MAX: u64 = (1 << 20) - 1;
const NATIVE_BERNOULLI_MAX_DEPTH: u32 = 32;
// One full shell of common-support margin: 2R <= K_max * scale, while the
// retained exact trace extends to (K_max + 1) * scale.
const NATIVE_CLIP_SCALE_MAX_RATIO: f64 = NATIVE_SAMPLER_K_MAX as f64 / 2.0;

pub(super) enum NativeF32Sample {
    Output(f32),
    RejectedSampler,
}

#[inline]
fn canonicalize_f32_output(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

enum NativeSamplerOutcome<T> {
    Value(T),
    Rejected,
}

struct NativeEntropy {
    bits: u64,
    available: u32,
}

impl NativeEntropy {
    #[inline]
    fn new() -> Self {
        Self {
            bits: 0,
            available: 0,
        }
    }

    #[inline]
    fn draw_u64(&mut self) -> Fallible<u64> {
        sample_from_uniform_bytes::<u64, 8>()
    }

    #[inline]
    fn draw_bits(&mut self, count: u32) -> Fallible<u64> {
        debug_assert!(count <= u64::BITS);
        if count == 0 {
            return Ok(0);
        }

        // Consume the reservoir in a fixed low-bit-first order, spanning source
        // words when necessary.  In particular, a 64-bit request consumes any
        // buffered suffix instead of silently drawing around it.
        let mut out = 0u64;
        let mut written = 0u32;
        while written < count {
            if self.available == 0 {
                self.bits = self.draw_u64()?;
                self.available = u64::BITS;
            }

            let take = self.available.min(count - written);
            let chunk = if take == u64::BITS {
                self.bits
            } else {
                self.bits & ((1u64 << take) - 1)
            };
            out |= chunk << written;

            if take == u64::BITS {
                self.bits = 0;
            } else {
                self.bits >>= take;
            }
            self.available -= take;
            written += take;
        }
        Ok(out)
    }

    #[inline]
    fn coin(&mut self) -> Fallible<bool> {
        Ok(self.draw_bits(1)? != 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
struct NativeUniform01 {
    // The prefix occupies the low 112 bits.  The unused high 16 bits pack the
    // random-prefix length and the fixed one-bit denominator shift used for a
    // lazy uniform on [0, 1/2).  The complete PSRN state is therefore 16 bytes.
    words: [u32; 4],
}

const NATIVE_UNIFORM_TOP_PREFIX_MASK: u32 = 0x0000_ffff;
const NATIVE_UNIFORM_BITS_SHIFT: u32 = 16;
const NATIVE_UNIFORM_BITS_MASK: u32 = 0x007f_0000;
const NATIVE_UNIFORM_SCALE_SHIFT_FLAG: u32 = 0x0080_0000;
const NATIVE_UNIFORM_METADATA_MASK: u32 =
    NATIVE_UNIFORM_BITS_MASK | NATIVE_UNIFORM_SCALE_SHIFT_FLAG;

impl NativeUniform01 {
    #[inline]
    fn new() -> Self {
        Self { words: [0; 4] }
    }

    #[inline]
    fn new_half() -> Self {
        let mut out = Self::new();
        out.words[3] = NATIVE_UNIFORM_SCALE_SHIFT_FLAG;
        out
    }

    #[inline]
    fn prefix(&self) -> u128 {
        self.words[0] as u128
            | ((self.words[1] as u128) << 32)
            | ((self.words[2] as u128) << 64)
            | (((self.words[3] & NATIVE_UNIFORM_TOP_PREFIX_MASK) as u128) << 96)
    }

    #[inline]
    fn set_prefix(&mut self, prefix: u128) {
        debug_assert!(prefix < (1u128 << NATIVE_SAMPLER_UNIFORM_MAX_BITS));
        let metadata = self.words[3] & NATIVE_UNIFORM_METADATA_MASK;
        self.words[0] = prefix as u32;
        self.words[1] = (prefix >> 32) as u32;
        self.words[2] = (prefix >> 64) as u32;
        self.words[3] = metadata | ((prefix >> 96) as u32 & NATIVE_UNIFORM_TOP_PREFIX_MASK);
    }

    #[inline]
    fn bits(&self) -> u32 {
        (self.words[3] & NATIVE_UNIFORM_BITS_MASK) >> NATIVE_UNIFORM_BITS_SHIFT
    }

    #[inline]
    fn scale_shift(&self) -> u32 {
        if self.words[3] & NATIVE_UNIFORM_SCALE_SHIFT_FLAG != 0 {
            1
        } else {
            0
        }
    }

    #[inline]
    fn set_bits(&mut self, bits: u32) {
        debug_assert!(bits <= NATIVE_SAMPLER_UNIFORM_MAX_BITS);
        self.words[3] =
            (self.words[3] & !NATIVE_UNIFORM_BITS_MASK) | (bits << NATIVE_UNIFORM_BITS_SHIFT);
    }

    #[inline]
    fn denominator_bits(&self) -> u32 {
        self.bits() + self.scale_shift()
    }

    #[inline]
    fn refine(&mut self, entropy: &mut NativeEntropy) -> Fallible<Option<()>> {
        self.refine_with_cap(entropy, NATIVE_SAMPLER_UNIFORM_MAX_BITS)
    }

    #[inline]
    fn refine_with_cap(
        &mut self,
        entropy: &mut NativeEntropy,
        max_bits: u32,
    ) -> Fallible<Option<()>> {
        let Some(bits) = self.bits().checked_add(NATIVE_UNIFORM_REFINE_BITS) else {
            return Ok(None);
        };
        if bits > max_bits {
            return Ok(None);
        }

        let random = entropy.draw_bits(NATIVE_UNIFORM_REFINE_BITS)? as u128;
        self.set_prefix((self.prefix() << NATIVE_UNIFORM_REFINE_BITS) | random);
        self.set_bits(bits);
        Ok(Some(()))
    }

    #[inline]
    fn greater_than_decided(&self, other: &Self) -> Option<bool> {
        let common = self.denominator_bits().max(other.denominator_bits());
        let self_lo = scaled_prefix(self.prefix(), false, self.denominator_bits(), common)?;
        let self_hi = scaled_prefix(self.prefix(), true, self.denominator_bits(), common)?;
        let other_lo = scaled_prefix(other.prefix(), false, other.denominator_bits(), common)?;
        let other_hi = scaled_prefix(other.prefix(), true, other.denominator_bits(), common)?;

        if self_lo >= other_hi {
            return Some(true);
        }
        if self_hi <= other_lo {
            return Some(false);
        }
        None
    }

    #[inline]
    fn greater_than(
        &mut self,
        other: &mut Self,
        entropy: &mut NativeEntropy,
    ) -> Fallible<Option<bool>> {
        loop {
            if let Some(result) = self.greater_than_decided(other) {
                return Ok(Some(result));
            }

            let self_denominator_bits = self.denominator_bits();
            let other_denominator_bits = other.denominator_bits();
            if self_denominator_bits == other_denominator_bits {
                if self.refine(entropy)?.is_none() {
                    return Ok(None);
                }
                if other.refine(entropy)?.is_none() {
                    return Ok(None);
                }
                continue;
            }

            let refined = if self_denominator_bits < other_denominator_bits {
                self.refine(entropy)?
            } else {
                other.refine(entropy)?
            };
            if refined.is_none() {
                return Ok(None);
            }
        }
    }
}

#[inline]
fn scaled_prefix(
    prefix: u128,
    add_one: bool,
    denominator_bits: u32,
    common_denominator_bits: u32,
) -> Option<u128> {
    let value = if add_one {
        prefix.checked_add(1)?
    } else {
        prefix
    };
    value.checked_shl(common_denominator_bits.checked_sub(denominator_bits)?)
}

// Exact Bernoulli(exp(-1/2)), using an early first-bit decomposition of
// the decreasing-chain construction in DFW Algorithm 2.
fn sample_bernoulli_exp_half_with(
    entropy: &mut NativeEntropy,
) -> Fallible<NativeSamplerOutcome<bool>> {
    if entropy.coin()? {
        return Ok(NativeSamplerOutcome::Value(true));
    }

    let mut accept_on_failure = false;
    let mut y = NativeUniform01::new_half();
    let mut depth = 0u32;

    loop {
        if depth >= NATIVE_BERNOULLI_MAX_DEPTH {
            return Ok(NativeSamplerOutcome::Rejected);
        }
        depth += 1;

        let mut u = NativeUniform01::new();

        match y.greater_than(&mut u, entropy)? {
            Some(true) => {}
            Some(false) => return Ok(NativeSamplerOutcome::Value(accept_on_failure)),
            None => return Ok(NativeSamplerOutcome::Rejected),
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

// DFW Algorithm 5, specialized to sigma = 1.
fn sample_k_with(entropy: &mut NativeEntropy) -> Fallible<NativeSamplerOutcome<u64>> {
    'restart: loop {
        match sample_bernoulli_exp_half_with(entropy)? {
            NativeSamplerOutcome::Value(true) => {}
            NativeSamplerOutcome::Value(false) => return Ok(NativeSamplerOutcome::Value(0)),
            NativeSamplerOutcome::Rejected => return Ok(NativeSamplerOutcome::Rejected),
        }

        match sample_bernoulli_exp_half_with(entropy)? {
            NativeSamplerOutcome::Value(true) => {}
            NativeSamplerOutcome::Value(false) => return Ok(NativeSamplerOutcome::Value(1)),
            NativeSamplerOutcome::Rejected => return Ok(NativeSamplerOutcome::Rejected),
        }

        let mut k = 2u64;

        loop {
            if k > NATIVE_SAMPLER_K_MAX {
                continue 'restart;
            }

            let mut t = (k - 1) * 2;

            while t != 0 {
                match sample_bernoulli_exp_half_with(entropy)? {
                    NativeSamplerOutcome::Value(true) => {}
                    NativeSamplerOutcome::Value(false) => continue 'restart,
                    NativeSamplerOutcome::Rejected => return Ok(NativeSamplerOutcome::Rejected),
                }
                t -= 1;
            }

            match sample_bernoulli_exp_half_with(entropy)? {
                NativeSamplerOutcome::Value(true) => {}
                NativeSamplerOutcome::Value(false) => return Ok(NativeSamplerOutcome::Value(k)),
                NativeSamplerOutcome::Rejected => return Ok(NativeSamplerOutcome::Rejected),
            }

            k += 1;
        }
    }
}

// sample_bernoulli_exp(-x) within [0, 1]
fn sample_bernoulli_exp_uniform_native(
    entropy: &mut NativeEntropy,
    x: &mut NativeUniform01,
) -> Fallible<NativeSamplerOutcome<bool>> {
    let mut y = NativeUniform01::new();
    let mut depth = 0u32;

    depth += 1;
    match x.greater_than(&mut y, entropy)? {
        Some(true) => {}
        Some(false) => return Ok(NativeSamplerOutcome::Value(true)),
        None => return Ok(NativeSamplerOutcome::Rejected),
    }

    let mut accept_on_failure = false;

    loop {
        if depth >= NATIVE_BERNOULLI_MAX_DEPTH {
            return Ok(NativeSamplerOutcome::Rejected);
        }
        depth += 1;

        let mut u = NativeUniform01::new();

        match y.greater_than(&mut u, entropy)? {
            Some(true) => {}
            Some(false) => return Ok(NativeSamplerOutcome::Value(accept_on_failure)),
            None => return Ok(NativeSamplerOutcome::Rejected),
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

#[inline]
fn uniform_less_than_half_decided(u: &NativeUniform01, x: &NativeUniform01) -> Option<bool> {
    let u_denominator = u.denominator_bits();
    let x_half_denominator = x.denominator_bits().checked_add(1)?;
    let common = u_denominator.max(x_half_denominator);

    let u_lo = scaled_prefix(u.prefix(), false, u_denominator, common)?;
    let u_hi = scaled_prefix(u.prefix(), true, u_denominator, common)?;
    let x_lo = scaled_prefix(x.prefix(), false, x_half_denominator, common)?;
    let x_hi = scaled_prefix(x.prefix(), true, x_half_denominator, common)?;

    if u_hi <= x_lo {
        return Some(true);
    }
    if u_lo >= x_hi {
        return Some(false);
    }
    None
}

fn uniform_less_than_half_native(
    entropy: &mut NativeEntropy,
    u: &mut NativeUniform01,
    x: &mut NativeUniform01,
) -> Fallible<Option<bool>> {
    loop {
        if let Some(result) = uniform_less_than_half_decided(u, x) {
            return Ok(Some(result));
        }

        let refined = if u.denominator_bits() <= x.denominator_bits().saturating_add(1) {
            u.refine(entropy)?
        } else {
            x.refine(entropy)?
        };
        if refined.is_none() {
            return Ok(None);
        }
    }
}

// DFW Algorithm 6 with parameters (x/2, x), realizing exp(-x^2/2).
fn sample_bernoulli_exp_half_x_squared_native(
    entropy: &mut NativeEntropy,
    x: &mut NativeUniform01,
) -> Fallible<NativeSamplerOutcome<bool>> {
    let mut accept_on_failure = true;
    let mut w: Option<NativeUniform01> = None;
    let mut depth = 0u32;

    loop {
        if depth >= NATIVE_BERNOULLI_MAX_DEPTH {
            return Ok(NativeSamplerOutcome::Rejected);
        }
        depth += 1;

        let mut u = NativeUniform01::new();

        let u_lt_w = match w.as_mut() {
            Some(w_prev) => w_prev.greater_than(&mut u, entropy)?,
            None => uniform_less_than_half_native(entropy, &mut u, x)?,
        };

        match u_lt_w {
            Some(true) => {}
            Some(false) => return Ok(NativeSamplerOutcome::Value(accept_on_failure)),
            None => return Ok(NativeSamplerOutcome::Rejected),
        }

        let mut v = NativeUniform01::new();
        match x.greater_than(&mut v, entropy)? {
            Some(true) => {}
            Some(false) => return Ok(NativeSamplerOutcome::Value(accept_on_failure)),
            None => return Ok(NativeSamplerOutcome::Rejected),
        }

        w = Some(u);
        accept_on_failure = !accept_on_failure;
    }
}

// Steps 3--4 of DFW Algorithm 7: exp(-k x) exp(-x^2/2).
fn accept_fraction_native(
    entropy: &mut NativeEntropy,
    k: u64,
    x: &mut NativeUniform01,
) -> Fallible<NativeSamplerOutcome<bool>> {
    let mut remaining = k;

    // peeling off bernoulli exp(-x) for x in [0, 1] k times
    while remaining != 0 {
        match sample_bernoulli_exp_uniform_native(entropy, x)? {
            NativeSamplerOutcome::Value(true) => {}
            NativeSamplerOutcome::Value(false) => return Ok(NativeSamplerOutcome::Value(false)),
            NativeSamplerOutcome::Rejected => return Ok(NativeSamplerOutcome::Rejected),
        }
        remaining -= 1;
    }

    sample_bernoulli_exp_half_x_squared_native(entropy, x)
}

fn snap_scale_up_to_f32(scale: f64) -> Fallible<f32> {
    debug_assert!(scale > 0.0 && scale.is_finite());

    if scale > f32::MAX as f64 {
        return fallible!(
            FailedFunction,
            "scale is too large to snap upward to a finite f32"
        );
    }

    let mut snapped = scale as f32;
    if snapped == 0.0 {
        return Ok(f32::from_bits(1));
    }
    if (snapped as f64) < scale {
        snapped = snapped.next_up_();
    }
    Ok(snapped)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct U160([u32; 5]);

impl U160 {
    const ZERO: Self = Self([0; 5]);

    #[inline]
    fn is_zero(self) -> bool {
        self.0 == [0; 5]
    }

    #[inline]
    fn cmp(self, other: Self) -> std::cmp::Ordering {
        self.0.into_iter().rev().cmp(other.0.into_iter().rev())
    }

    #[inline]
    fn from_u128(value: u128) -> Self {
        Self([
            value as u32,
            (value >> 32) as u32,
            (value >> 64) as u32,
            (value >> 96) as u32,
            0,
        ])
    }

    fn from_u128_shifted(value: u128, shift: u32) -> Option<Self> {
        if value == 0 {
            return Some(Self::ZERO);
        }
        let bit_len = u128::BITS - value.leading_zeros();
        if shift >= 160 || bit_len.checked_add(shift)? > 160 {
            return None;
        }

        let src = [
            value as u32,
            (value >> 32) as u32,
            (value >> 64) as u32,
            (value >> 96) as u32,
        ];
        let word_shift = (shift / 32) as usize;
        let bit_shift = shift % 32;
        let mut out = [0u32; 5];

        for (idx, word) in src.into_iter().enumerate() {
            if word == 0 {
                continue;
            }
            let dst = idx + word_shift;
            if dst >= out.len() {
                return None;
            }
            out[dst] |= word << bit_shift;
            if bit_shift != 0 {
                let carry = word >> (32 - bit_shift);
                if carry != 0 {
                    let carry_dst = dst + 1;
                    if carry_dst >= out.len() {
                        return None;
                    }
                    out[carry_dst] |= carry;
                }
            }
        }
        Some(Self(out))
    }

    fn mul_u128_u32(value: u128, multiplier: u32) -> Option<Self> {
        let src = [
            value as u32,
            (value >> 32) as u32,
            (value >> 64) as u32,
            (value >> 96) as u32,
        ];
        let mut out = [0u32; 5];
        let mut carry = 0u64;
        for idx in 0..4 {
            let product = src[idx] as u64 * multiplier as u64 + carry;
            out[idx] = product as u32;
            carry = product >> 32;
        }
        out[4] = carry as u32;
        Some(Self(out))
    }

    fn checked_add(self, other: Self) -> Option<Self> {
        let mut out = [0u32; 5];
        let mut carry = 0u64;
        for idx in 0..5 {
            let sum = self.0[idx] as u64 + other.0[idx] as u64 + carry;
            out[idx] = sum as u32;
            carry = sum >> 32;
        }
        (carry == 0).then_some(Self(out))
    }

    fn checked_sub(self, other: Self) -> Option<Self> {
        if self.cmp(other).is_lt() {
            return None;
        }
        let mut out = [0u32; 5];
        let mut borrow = 0u64;
        for idx in 0..5 {
            let lhs = self.0[idx] as u64;
            let rhs = other.0[idx] as u64 + borrow;
            if lhs >= rhs {
                out[idx] = (lhs - rhs) as u32;
                borrow = 0;
            } else {
                out[idx] = ((1u64 << 32) + lhs - rhs) as u32;
                borrow = 1;
            }
        }
        debug_assert_eq!(borrow, 0);
        Some(Self(out))
    }

    #[inline]
    fn checked_add_u32(self, value: u32) -> Option<Self> {
        self.checked_add(Self([value, 0, 0, 0, 0]))
    }

    #[inline]
    fn checked_sub_one(self) -> Option<Self> {
        self.checked_sub(Self([1, 0, 0, 0, 0]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct I160(U160);

// The profile proves every magnitude is below 2^157.  Bit 159 stores the sign,
// leaving bits 157 and 158 as checked headroom in the same five-word object.
const I160_SIGN_MASK: u32 = 1u32 << 31;

impl I160 {
    fn new(negative: bool, mut magnitude: U160) -> Option<Self> {
        if magnitude.0[4] & I160_SIGN_MASK != 0 {
            return None;
        }
        if negative && !magnitude.is_zero() {
            magnitude.0[4] |= I160_SIGN_MASK;
        }
        Some(Self(magnitude))
    }

    #[inline]
    fn negative(self) -> bool {
        self.0.0[4] & I160_SIGN_MASK != 0
    }

    #[inline]
    fn magnitude(self) -> U160 {
        let mut magnitude = self.0;
        magnitude.0[4] &= !I160_SIGN_MASK;
        magnitude
    }

    fn add_signed(self, negative: bool, magnitude: U160) -> Option<Self> {
        let lhs = self.magnitude();
        if magnitude.is_zero() {
            return Some(self);
        }
        if lhs.is_zero() {
            return Self::new(negative, magnitude);
        }
        if self.negative() == negative {
            return Self::new(self.negative(), lhs.checked_add(magnitude)?);
        }
        match lhs.cmp(magnitude) {
            std::cmp::Ordering::Greater => Self::new(self.negative(), lhs.checked_sub(magnitude)?),
            std::cmp::Ordering::Less => Self::new(negative, magnitude.checked_sub(lhs)?),
            std::cmp::Ordering::Equal => Self::new(false, U160::ZERO),
        }
    }

    #[inline]
    fn clamp_magnitude(self, limit: U160) -> Option<Self> {
        if self.magnitude().cmp(limit).is_gt() {
            Self::new(self.negative(), limit)
        } else {
            Some(self)
        }
    }
}

#[derive(Clone, Copy)]
struct BinaryDyadic {
    negative: bool,
    mantissa: u128,
    exponent: i32,
}

fn finite_f64_to_binary_dyadic(value: f64) -> BinaryDyadic {
    debug_assert!(value.is_finite());
    let bits = value.to_bits();
    let negative = bits >> 63 != 0;
    let raw_exponent = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1u64 << 52) - 1);
    let (mantissa, exponent) = if raw_exponent == 0 {
        (fraction as u128, -1074)
    } else {
        (((1u64 << 52) | fraction) as u128, raw_exponent - 1023 - 52)
    };
    BinaryDyadic {
        negative,
        mantissa,
        exponent,
    }
}

#[derive(Clone, Copy)]
struct F32ScaleParts {
    mantissa: u32,
    exponent: i32,
}

fn positive_f32_scale_parts(value: f32) -> F32ScaleParts {
    debug_assert!(value.is_finite() && value > 0.0);
    let bits = value.to_bits();
    let raw_exponent = ((bits >> 23) & 0xff) as i32;
    let fraction = bits & ((1u32 << 23) - 1);
    if raw_exponent == 0 {
        F32ScaleParts {
            mantissa: fraction,
            exponent: -149,
        }
    } else {
        F32ScaleParts {
            mantissa: (1u32 << 23) | fraction,
            exponent: raw_exponent - 127 - 23,
        }
    }
}

fn rounded_f64_grid_index(value: f64, grid_exponent: i32) -> Option<I160> {
    let dyadic = finite_f64_to_binary_dyadic(value);
    if dyadic.mantissa == 0 {
        return I160::new(false, U160::ZERO);
    }

    let shift = dyadic.exponent.checked_sub(grid_exponent)?;
    let magnitude = if shift >= 0 {
        U160::from_u128_shifted(dyadic.mantissa, shift as u32)?
    } else {
        let right = (-shift) as u32;
        if right >= u128::BITS {
            U160::ZERO
        } else {
            let quotient = dyadic.mantissa >> right;
            let remainder = dyadic.mantissa & ((1u128 << right) - 1);
            let half = 1u128 << (right - 1);
            let increment = remainder > half || (remainder == half && quotient & 1 == 1);
            U160::from_u128(quotient + u128::from(increment))
        }
    };
    I160::new(dyadic.negative, magnitude)
}

fn floor_positive_f64_grid_index(value: f64, grid_exponent: i32) -> Option<U160> {
    debug_assert!(value.is_finite() && value >= 0.0);
    let dyadic = finite_f64_to_binary_dyadic(value);
    if dyadic.mantissa == 0 {
        return Some(U160::ZERO);
    }
    let shift = dyadic.exponent.checked_sub(grid_exponent)?;
    if shift >= 0 {
        U160::from_u128_shifted(dyadic.mantissa, shift as u32)
    } else {
        let right = (-shift) as u32;
        if right >= u128::BITS {
            Some(U160::ZERO)
        } else {
            Some(U160::from_u128(dyadic.mantissa >> right))
        }
    }
}

// Exact uniform draw on a public range.  Rejected proposals depend only on the
// public scale mantissa and fresh entropy, so this is ordinary sampler rejection
// and does not condition the emitted law.
fn sample_below_u32(entropy: &mut NativeEntropy, upper: u32) -> Fallible<u32> {
    debug_assert!(upper > 0);
    if upper == 1 {
        return Ok(0);
    }
    let bits = u32::BITS - (upper - 1).leading_zeros();
    loop {
        let candidate = entropy.draw_bits(bits)? as u32;
        if candidate < upper {
            return Ok(candidate);
        }
    }
}

fn refine_uniform_to_profile(x: &mut NativeUniform01, entropy: &mut NativeEntropy) -> Fallible<()> {
    debug_assert_eq!(x.scale_shift(), 0);
    while x.bits() < NATIVE_SAMPLER_UNIFORM_MAX_BITS {
        if x.refine_with_cap(entropy, NATIVE_SAMPLER_UNIFORM_MAX_BITS)?
            .is_none()
        {
            return fallible!(
                FailedFunction,
                "accepted partial uniform cannot reach the declared 112-bit profile"
            );
        }
    }
    Ok(())
}

fn accepted_noise_base(scale_mantissa: u32, k: u64, prefix: u128, subcell: u32) -> Option<U160> {
    let mut out = U160::mul_u128_u32(prefix, scale_mantissa)?;
    let shell = (scale_mantissa as u64).checked_mul(k)?;
    out = out.checked_add(U160::from_u128_shifted(
        shell as u128,
        NATIVE_SAMPLER_UNIFORM_MAX_BITS,
    )?)?;
    out.checked_add_u32(subcell)
}

#[derive(Clone, Copy)]
struct GridMagnitudeBin {
    // Order fields so the transient representation is compact on common ABIs.
    fraction_bin: u64,
    integer: U160,
}

fn normalize_grid_bin(
    base: I160,
    noise_negative: bool,
    fraction_bin: u64,
) -> Option<(bool, GridMagnitudeBin)> {
    if base.magnitude().is_zero() {
        return Some((
            noise_negative,
            GridMagnitudeBin {
                fraction_bin,
                integer: U160::ZERO,
            },
        ));
    }

    if base.negative() == noise_negative {
        return Some((
            base.negative(),
            GridMagnitudeBin {
                fraction_bin,
                integer: base.magnitude(),
            },
        ));
    }

    Some((
        base.negative(),
        GridMagnitudeBin {
            fraction_bin: !fraction_bin,
            integer: base.magnitude().checked_sub_one()?,
        },
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct U224([u32; 7]);

impl U224 {
    const ZERO: Self = Self([0; 7]);

    fn from_u160_shifted(value: U160, shift: u32) -> Option<Self> {
        if value.is_zero() {
            return Some(Self::ZERO);
        }
        if shift >= 224 {
            return None;
        }
        let word_shift = (shift / 32) as usize;
        let bit_shift = shift % 32;
        let mut out = [0u32; 7];
        for (idx, word) in value.0.into_iter().enumerate() {
            if word == 0 {
                continue;
            }
            let dst = idx + word_shift;
            if dst >= out.len() {
                return None;
            }
            out[dst] |= word << bit_shift;
            if bit_shift != 0 {
                let carry = word >> (32 - bit_shift);
                if carry != 0 {
                    let carry_dst = dst + 1;
                    if carry_dst >= out.len() {
                        return None;
                    }
                    out[carry_dst] |= carry;
                }
            }
        }
        Some(Self(out))
    }

    fn from_completion_midpoint(bin: GridMagnitudeBin) -> Option<Self> {
        // integer + (u + 1/2) / 2^64 =
        // [integer * 2^65 + (2u + 1)] / 2^65.
        let mut out = Self::from_u160_shifted(bin.integer, NATIVE_COMPLETION_BITS + 1)?;
        let low = (bin.fraction_bin << 1) | 1;
        out.0[0] |= low as u32;
        out.0[1] |= (low >> 32) as u32;
        out.0[2] |= (bin.fraction_bin >> 63) as u32;
        Some(out)
    }

    #[inline]
    fn is_zero(self) -> bool {
        self.0 == [0; 7]
    }

    fn bit_len(self) -> u32 {
        for idx in (0..7).rev() {
            let word = self.0[idx];
            if word != 0 {
                return idx as u32 * 32 + (32 - word.leading_zeros());
            }
        }
        0
    }

    fn bit(self, index: u32) -> bool {
        if index >= 224 {
            return false;
        }
        let word = (index / 32) as usize;
        let offset = index % 32;
        (self.0[word] >> offset) & 1 != 0
    }

    fn any_bits_below(self, end: u32) -> bool {
        if end == 0 {
            return false;
        }
        let full_words = (end / 32).min(7) as usize;
        if self.0[..full_words].iter().any(|word| *word != 0) {
            return true;
        }
        if full_words >= 7 {
            return false;
        }
        let rem = end % 32;
        rem != 0 && (self.0[full_words] & ((1u32 << rem) - 1)) != 0
    }

    fn shr_to_u32(self, shift: u32) -> Option<u32> {
        if shift >= 224 {
            return Some(0);
        }
        let word = (shift / 32) as usize;
        let offset = shift % 32;
        let mut value = (self.0[word] as u64) >> offset;
        if offset != 0 && word + 1 < 7 {
            value |= (self.0[word + 1] as u64) << (32 - offset);
        }
        let high_start = if offset == 0 { word + 1 } else { word + 2 };
        for idx in high_start..7 {
            if self.0[idx] != 0 {
                return None;
            }
        }
        (value <= u32::MAX as u64).then_some(value as u32)
    }

    fn shl_to_u32(self, shift: u32) -> Option<u32> {
        if self.0[1..].iter().any(|word| *word != 0) {
            return None;
        }
        let shifted = (self.0[0] as u64).checked_shl(shift)?;
        (shifted <= u32::MAX as u64).then_some(shifted as u32)
    }

    fn rounded_coefficient(self, shift: i32) -> Option<u32> {
        if shift <= 0 {
            return self.shl_to_u32((-shift) as u32);
        }
        let shift = shift as u32;
        let quotient = self.shr_to_u32(shift)?;
        let half = self.bit(shift - 1);
        let sticky = self.any_bits_below(shift - 1);
        let increment = half && (sticky || quotient & 1 == 1);
        quotient.checked_add(if increment { 1 } else { 0 })
    }
}

fn round_dyadic_to_f32(negative: bool, mantissa: U224, exponent: i32) -> Option<f32> {
    if mantissa.is_zero() {
        return Some(0.0);
    }

    let top = mantissa.bit_len() as i32 - 1;
    let mut unbiased_exponent = exponent.checked_add(top)?;
    if unbiased_exponent > 127 {
        return Some(if negative {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        });
    }

    let bits = if unbiased_exponent >= -126 {
        let mut coefficient = mantissa.rounded_coefficient(top - 23)?;
        if coefficient == 1 << 24 {
            coefficient >>= 1;
            unbiased_exponent += 1;
        }
        if unbiased_exponent > 127 {
            return Some(if negative {
                f32::NEG_INFINITY
            } else {
                f32::INFINITY
            });
        }
        debug_assert!(((1 << 23)..(1 << 24)).contains(&coefficient));
        (((unbiased_exponent + 127) as u32) << 23) | (coefficient - (1 << 23))
    } else {
        let coefficient = mantissa.rounded_coefficient(-149 - exponent)?;
        if coefficient == 0 {
            return Some(0.0);
        }
        if coefficient >= 1 << 23 {
            // Rounding promoted the largest subnormal interval to min normal.
            1 << 23
        } else {
            coefficient
        }
    };

    let bits = if negative { bits | (1 << 31) } else { bits };
    Some(f32::from_bits(bits))
}

fn round_exact_grid_magnitude_to_f32(
    negative: bool,
    magnitude: U160,
    grid_exponent: i32,
) -> Option<f32> {
    round_dyadic_to_f32(
        negative,
        U224::from_u160_shifted(magnitude, 0)?,
        grid_exponent,
    )
}

fn round_grid_bin_midpoint_to_f32(
    negative: bool,
    bin: GridMagnitudeBin,
    grid_exponent: i32,
) -> Option<f32> {
    round_dyadic_to_f32(
        negative,
        U224::from_completion_midpoint(bin)?,
        grid_exponent.checked_sub((NATIVE_COMPLETION_BITS + 1) as i32)?,
    )
}

/// Public, prevalidated fixed-width profile for one DP-SGD Gaussian release family.
///
/// Scale snapping, completion-grid admission, clipping alignment, and all
/// fixed-width radius checks are performed once before private coordinates are
/// processed.  The resulting object is read-only and may be shared across GPU
/// threads.  The profile is defined only for positive Gaussian scales.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NativeF32Profile {
    snapped_scale: f32,
    scale_mantissa: u32,
    grid_exponent: i32,
    clip: f64,
    clip_index: U160,
}

impl NativeF32Profile {
    pub(super) fn new(scale: f64, clip: f64) -> Fallible<Self> {
        if !scale.is_finite() || scale <= 0.0 {
            return fallible!(FailedFunction, "scale must be finite and positive");
        }
        if !clip.is_finite() || clip < 0.0 {
            return fallible!(FailedFunction, "clip must be finite and nonnegative");
        }

        let scale = snap_scale_up_to_f32(scale)?;
        let scale_parts = positive_f32_scale_parts(scale);
        if scale_parts.exponent > 26 {
            return fallible!(
                FailedFunction,
                "scale is outside the fixed-width DP-SGD binary32 profile"
            );
        }

        let threshold_bits =
            (scale_parts.exponent + 150 - NATIVE_SAMPLER_UNIFORM_MAX_BITS as i32).max(0) as u32;
        if threshold_bits > NATIVE_COMPLETION_BITS {
            return fallible!(
                FailedFunction,
                "binary32 thresholds exceed the 64-bit completion grid"
            );
        }

        if clip > NATIVE_CLIP_SCALE_MAX_RATIO * f64::from(scale) {
            return fallible!(
                FailedFunction,
                "range must be <= 524287.5 * the snapped scale"
            );
        }

        let grid_exponent = scale_parts.exponent - NATIVE_SAMPLER_UNIFORM_MAX_BITS as i32;
        let Some(clip_index) = floor_positive_f64_grid_index(clip, grid_exponent) else {
            return fallible!(FailedFunction, "aligned clipping index exceeds 160 bits");
        };

        Ok(Self {
            snapped_scale: scale,
            scale_mantissa: scale_parts.mantissa,
            grid_exponent,
            clip,
            clip_index,
        })
    }

    /// Sample one private coordinate entirely with fixed-width native arithmetic.
    ///
    /// A sampler-resource rejection is retried per coordinate by the GPU wrapper
    /// and is charged by the pure-RDP resource term.  There is no finalizer
    /// rejection and no per-sample arbitrary-precision path.
    pub(super) fn sample(&self, mu: f64) -> Fallible<NativeF32Sample> {
        if !mu.is_finite() {
            return fallible!(FailedFunction, "location must be finite");
        }

        if self.clip_index.is_zero() {
            return Ok(NativeF32Sample::Output(0.0));
        }

        let clamped_mu = mu.max(-self.clip).min(self.clip);
        let Some(center) = rounded_f64_grid_index(clamped_mu, self.grid_exponent)
            .and_then(|center| center.clamp_magnitude(self.clip_index))
        else {
            return fallible!(FailedFunction, "aligned center index exceeds 160 bits");
        };

        let mut entropy = NativeEntropy::new();
        loop {
            let k = match sample_k_with(&mut entropy)? {
                NativeSamplerOutcome::Value(k) => k,
                NativeSamplerOutcome::Rejected => {
                    return Ok(NativeF32Sample::RejectedSampler);
                }
            };

            let mut x = NativeUniform01::new();
            match accept_fraction_native(&mut entropy, k, &mut x)? {
                NativeSamplerOutcome::Value(true) => {}
                NativeSamplerOutcome::Value(false) => continue,
                NativeSamplerOutcome::Rejected => {
                    return Ok(NativeF32Sample::RejectedSampler);
                }
            }

            // Complete the aligned mantissa split.  Conditional on the accepted
            // 112-bit prefix, the unread suffix is an independent uniform.  We
            // may therefore resample that suffix with fresh entropy: `j` is
            // uniform below the scale mantissa and `completion_index` selects
            // a uniform 64-bit completion bin.
            refine_uniform_to_profile(&mut x, &mut entropy)?;
            let j = sample_below_u32(&mut entropy, self.scale_mantissa)?;
            let completion_index = entropy.draw_bits(NATIVE_COMPLETION_BITS)?;
            let sign_negative = entropy.coin()?;

            let Some(aligned_noise_integer) =
                accepted_noise_base(self.scale_mantissa, k, x.prefix(), j)
            else {
                return fallible!(FailedFunction, "aligned noise index exceeds 160 bits");
            };
            let Some(aligned_affine_integer) =
                center.add_signed(sign_negative, aligned_noise_integer)
            else {
                return fallible!(FailedFunction, "aligned affine index exceeds 160 bits");
            };

            // Normalize A + S*W to a sign and a nonnegative completion bin
            // before the fixed-work clipping and direct binary32 conversion.
            let Some((output_negative, magnitude_bin)) =
                normalize_grid_bin(aligned_affine_integer, sign_negative, completion_index)
            else {
                return fallible!(FailedFunction, "failed to normalize aligned unit interval");
            };

            let output = if magnitude_bin.integer.cmp(self.clip_index).is_ge() {
                round_exact_grid_magnitude_to_f32(
                    output_negative,
                    self.clip_index,
                    self.grid_exponent,
                )
            } else {
                round_grid_bin_midpoint_to_f32(output_negative, magnitude_bin, self.grid_exponent)
            };
            let Some(output) = output else {
                return fallible!(
                    FailedFunction,
                    "exact binary32 conversion exceeded its static profile"
                );
            };
            return Ok(NativeF32Sample::Output(canonicalize_f32_output(output)));
        }
    }
}
