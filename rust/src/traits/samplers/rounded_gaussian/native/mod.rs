use crate::{error::Fallible, traits::NextFloat, traits::samplers::sample_from_uniform_bytes};

#[cfg(test)]
mod test;

const NATIVE_UNIFORM_REFINE_BITS: u32 = 4;
const NATIVE_UNIFORM_MAX_BITS: u32 = 120;

pub(super) enum NativeF32Sample {
    Output(f32),
    RejectedComb,
    ResourceLimit,
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
        if count == u64::BITS {
            return self.draw_u64();
        }

        if self.available < count {
            self.bits = self.draw_u64()?;
            self.available = u64::BITS;
        }

        let mask = (1u64 << count) - 1;
        let out = self.bits & mask;
        self.bits >>= count;
        self.available -= count;
        Ok(out)
    }

    #[inline]
    fn coin(&mut self) -> Fallible<bool> {
        Ok(self.draw_bits(1)? != 0)
    }
}

#[derive(Clone, Copy)]
struct NativeUniform01 {
    prefix: u128,
    bits: u32,
}

impl NativeUniform01 {
    #[inline]
    fn new() -> Self {
        Self { prefix: 0, bits: 0 }
    }

    #[inline]
    fn refine(&mut self, entropy: &mut NativeEntropy) -> Fallible<Option<()>> {
        let Some(bits) = self.bits.checked_add(NATIVE_UNIFORM_REFINE_BITS) else {
            return Ok(None);
        };
        if bits > NATIVE_UNIFORM_MAX_BITS {
            return Ok(None);
        }

        let random = entropy.draw_bits(NATIVE_UNIFORM_REFINE_BITS)? as u128;
        self.prefix = (self.prefix << NATIVE_UNIFORM_REFINE_BITS) | random;
        self.bits = bits;
        Ok(Some(()))
    }

    #[inline]
    fn interval_f64(&self) -> (f64, f64) {
        if self.bits == 0 {
            return (0.0, 1.0);
        }

        let scale = f64::from_bits(((1023 - self.bits as u64) << 52) as u64);
        let lower = ((self.prefix as f64) * scale).next_down_().max(0.0);
        let upper = (((self.prefix + 1) as f64) * scale).next_up_().min(1.0);
        (lower, upper)
    }

    #[inline]
    fn greater_than_decided(&self, other: &Self) -> Option<bool> {
        if self.bits == 0 || other.bits == 0 {
            return None;
        }

        // if denominators match then compare prefixes
        if self.bits == other.bits {
            if self.prefix > other.prefix {
                return Some(true);
            }
            if self.prefix < other.prefix {
                return Some(false);
            }
            return None;
        }

        let bits = self.bits.max(other.bits);
        let self_shift = bits - self.bits;
        let other_shift = bits - other.bits;

        let self_lo = self.prefix << self_shift;
        let self_hi = (self.prefix + 1) << self_shift;
        let other_lo = other.prefix << other_shift;
        let other_hi = (other.prefix + 1) << other_shift;

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

            if self.bits == other.bits {
                if self.refine(entropy)?.is_none() {
                    return Ok(None);
                }
                if other.refine(entropy)?.is_none() {
                    return Ok(None);
                }
                continue;
            }

            let refined = if self.bits < other.bits {
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

// Algorithm 4 https://arxiv.org/pdf/2008.03855
fn sample_bernoulli_exp_half_with(entropy: &mut NativeEntropy) -> Fallible<Option<bool>> {
    if entropy.coin()? {
        return Ok(Some(true));
    }

    let mut accept_on_failure = false;
    let mut y = NativeUniform01 { prefix: 0, bits: 1 };

    loop {
        let mut u = NativeUniform01::new();

        match y.greater_than(&mut u, entropy)? {
            Some(true) => {}
            Some(false) => return Ok(Some(accept_on_failure)),
            None => return Ok(None),
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

// Algorithm 5: https://arxiv.org/pdf/2008.03855
fn sample_k_with(entropy: &mut NativeEntropy) -> Fallible<Option<u64>> {
    'restart: loop {
        match sample_bernoulli_exp_half_with(entropy)? {
            Some(true) => {}
            Some(false) => return Ok(Some(0)),
            None => return Ok(None),
        }

        match sample_bernoulli_exp_half_with(entropy)? {
            Some(true) => {}
            Some(false) => return Ok(Some(1)),
            None => return Ok(None),
        }

        let mut k = 2u64;

        loop {
            let Some(mut t) = k.checked_sub(1).and_then(|value| value.checked_mul(2)) else {
                return Ok(None);
            };

            while t != 0 {
                match sample_bernoulli_exp_half_with(entropy)? {
                    Some(true) => {}
                    Some(false) => continue 'restart,
                    None => return Ok(None),
                }
                t -= 1;
            }

            match sample_bernoulli_exp_half_with(entropy)? {
                Some(true) => {}
                Some(false) => return Ok(Some(k)),
                None => return Ok(None),
            }

            let Some(next) = k.checked_add(1) else {
                return Ok(None);
            };
            k = next;
        }
    }
}

// sample_bernoulli_exp(-x) within [0, 1]
fn sample_bernoulli_exp_uniform_native(
    entropy: &mut NativeEntropy,
    x: &mut NativeUniform01,
) -> Fallible<Option<bool>> {
    let mut y = NativeUniform01::new();

    match x.greater_than(&mut y, entropy)? {
        Some(true) => {}
        Some(false) => return Ok(Some(true)),
        None => return Ok(None),
    }

    let mut accept_on_failure = false;

    loop {
        let mut u = NativeUniform01::new();

        match y.greater_than(&mut u, entropy)? {
            Some(true) => {}
            Some(false) => return Ok(Some(accept_on_failure)),
            None => return Ok(None),
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

#[inline]
fn shifted_u128_words(value: u128, shift: u32) -> [u64; 4] {
    debug_assert!(shift < u128::BITS);

    let src = [value as u64, (value >> u64::BITS) as u64];
    let word_shift = (shift / u64::BITS) as usize;
    let bit_shift = shift % u64::BITS;
    let mut out = [0u64; 4];

    for (idx, word) in src.into_iter().enumerate() {
        let out_idx = idx + word_shift;
        if out_idx >= out.len() {
            continue;
        }

        out[out_idx] |= word << bit_shift;

        if bit_shift != 0 && out_idx + 1 < out.len() {
            out[out_idx + 1] |= word >> (u64::BITS - bit_shift);
        }
    }

    out
}

#[inline]
fn shifted_bit_len(value: u128, shift: u32) -> u32 {
    if value == 0 {
        0
    } else {
        u128::BITS - value.leading_zeros() + shift
    }
}

#[inline]
fn cmp_shifted_u128(lhs: u128, lhs_shift: u32, rhs: u128, rhs_shift: u32) -> std::cmp::Ordering {
    if shifted_bit_len(lhs, lhs_shift).max(shifted_bit_len(rhs, rhs_shift)) <= u128::BITS {
        return (lhs << lhs_shift).cmp(&(rhs << rhs_shift));
    }

    let lhs = shifted_u128_words(lhs, lhs_shift);
    let rhs = shifted_u128_words(rhs, rhs_shift);

    lhs.into_iter().rev().cmp(rhs.into_iter().rev())
}

#[inline]
fn uniform_less_than_half_decided(u: &NativeUniform01, x: &NativeUniform01) -> Option<bool> {
    if cmp_shifted_u128(u.prefix + 1, x.bits + 1, x.prefix, u.bits).is_lt() {
        return Some(true);
    }

    if cmp_shifted_u128(u.prefix, x.bits + 1, x.prefix + 1, u.bits).is_gt() {
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

        let refined = if u.bits <= x.bits.saturating_add(1) {
            u.refine(entropy)?
        } else {
            x.refine(entropy)?
        };
        if refined.is_none() {
            return Ok(None);
        }
    }
}

// Algorithm 6 https://arxiv.org/pdf/2008.03855
fn sample_bernoulli_exp_half_x_squared_native(
    entropy: &mut NativeEntropy,
    x: &mut NativeUniform01,
) -> Fallible<Option<bool>> {
    let mut accept_on_failure = true;
    let mut w: Option<NativeUniform01> = None;

    loop {
        let mut u = NativeUniform01::new();

        let u_lt_w = match w.as_mut() {
            Some(w_prev) => w_prev.greater_than(&mut u, entropy)?,
            None => uniform_less_than_half_native(entropy, &mut u, x)?,
        };

        match u_lt_w {
            Some(true) => {}
            Some(false) => return Ok(Some(accept_on_failure)),
            None => return Ok(None),
        }

        let mut v = NativeUniform01::new();
        match x.greater_than(&mut v, entropy)? {
            Some(true) => {}
            Some(false) => return Ok(Some(accept_on_failure)),
            None => return Ok(None),
        }

        w = Some(u);
        accept_on_failure = !accept_on_failure;
    }
}

// Algorithm 7 https://arxiv.org/pdf/2008.03855
fn accept_fraction_native(
    entropy: &mut NativeEntropy,
    k: u64,
    x: &mut NativeUniform01,
) -> Fallible<Option<bool>> {
    let mut remaining = k;

    // peeling off bernoulli exp(-x) for x in [0, 1] k times
    while remaining != 0 {
        match sample_bernoulli_exp_uniform_native(entropy, x)? {
            Some(true) => {}
            Some(false) => return Ok(Some(false)),
            None => return Ok(None),
        }
        remaining -= 1;
    }


    sample_bernoulli_exp_half_x_squared_native(entropy, x)
}

fn down_add(a: f64, b: f64) -> f64 {
    (a + b).next_down_()
}

fn up_add(a: f64, b: f64) -> f64 {
    (a + b).next_up_()
}

fn down_sub(a: f64, b: f64) -> f64 {
    (a - b).next_down_()
}

fn up_sub(a: f64, b: f64) -> f64 {
    (a - b).next_up_()
}

fn down_mul(a: f64, b: f64) -> f64 {
    (a * b).next_down_()
}

fn up_mul(a: f64, b: f64) -> f64 {
    (a * b).next_up_()
}

fn u64_interval_f64(x: u64) -> (f64, f64) {
    let out = x as f64;
    if x <= (1u64 << f64::MANTISSA_DIGITS) {
        (out, out)
    } else {
        (out.next_down_(), out.next_up_())
    }
}

enum F32CellCertification {
    Output(f32),
    Unknown,
}

fn certify_real_affine_rounds_to_f32(
    mu: f64,
    scale: f64,
    clip: Option<f64>,
    negative: bool,
    k: u64,
    x: &NativeUniform01,
) -> F32CellCertification {
    let (k_lo, k_hi) = u64_interval_f64(k);
    let (x_lo, x_hi) = x.interval_f64();
    let mag_lo = down_add(k_lo, x_lo);
    let mag_hi = up_add(k_hi, x_hi);

    let (lo, hi) = if negative {
        (
            down_sub(mu, up_mul(scale, mag_hi)),
            up_sub(mu, down_mul(scale, mag_lo)),
        )
    } else {
        (
            down_add(mu, down_mul(scale, mag_lo)),
            up_add(mu, up_mul(scale, mag_hi)),
        )
    };

    let (lo, hi) = match clip {
        Some(range) => (lo.max(-range).min(range), hi.max(-range).min(range)),
        None => (lo, hi),
    };

    let lo = lo as f32;
    let hi = hi as f32;
    if lo != hi {
        return F32CellCertification::Unknown;
    }
    F32CellCertification::Output(lo)
}

/// Sample with f64 input parameters and try to certify the exact real affine
/// result directly into an extended f32 output cell.
///
/// This does not construct a floating-point noise value and then add it to
/// `mu`. The sampled Karney variate remains a lazy real interval, and this
/// routine returns only after proving that the real value
///
///     mu +/- scale * (k + x)
///
/// rounds to one f32 cell, including overflow cells represented by infinities.
///
/// Failure to certify before the native PSRN cap rejects the accepted trace as
/// an unresolved rounding-boundary comb. This slightly conditions the target
/// law, but avoids crossing into exact rational finalization.
pub(super) fn sample_f64_to_f32_clipped(
    mu: f64,
    scale: f64,
    clip: Option<f64>,
) -> Fallible<NativeF32Sample> {
    if !mu.is_finite() || !scale.is_finite() {
        return fallible!(FailedFunction, "mu and scale must be finite");
    }
    if scale < 0.0 {
        return fallible!(FailedFunction, "scale must be nonnegative");
    }
    if let Some(clip) = clip
        && (!clip.is_finite() || clip < 0.0)
    {
        return fallible!(FailedFunction, "clip must be finite and nonnegative");
    }
    if scale == 0.0 {
        let out = clip.map(|range| mu.max(-range).min(range)).unwrap_or(mu) as f32;
        return Ok(NativeF32Sample::Output(out));
    }

    let mut entropy = NativeEntropy::new();
    loop {
        let Some(k) = sample_k_with(&mut entropy)? else {
            return Ok(NativeF32Sample::ResourceLimit);
        };
        let mut x = NativeUniform01::new();

        match accept_fraction_native(&mut entropy, k, &mut x)? {
            Some(true) => {}
            Some(false) => continue,
            None => return Ok(NativeF32Sample::ResourceLimit),
        }

        let negative = entropy.coin()?;

        loop {
            match certify_real_affine_rounds_to_f32(mu, scale, clip, negative, k, &x) {
                F32CellCertification::Output(out) => return Ok(NativeF32Sample::Output(out)),
                F32CellCertification::Unknown => {}
            }

            if x.refine(&mut entropy)?.is_none() {
                return Ok(NativeF32Sample::RejectedComb);
            }
        }
    }
}
