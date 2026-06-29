use crate::{
    error::Fallible,
    traits::{NextFloat, samplers::sample_from_uniform_bytes},
};

#[cfg(test)]
mod test;

const NATIVE_UNIFORM_REFINE_BITS: u32 = 4;

// Finalization budget for the PSRN fractional prefix.
//
// This native path has two finite-budget events with privacy-accounting cost:
// the sampler-side tail from representing K in a native integer, and the
// finalization comb from giving up near f32 cell boundaries. K is left at the
// natural u64 sampler bound because its half-Gaussian tail is already far below
// any practical comb term. The useful arithmetic budget is spent on the PSRN
// prefix instead.
//
// With a snapped f32 scale, m_sigma has at most 24 bits. At b = 96 the upper
// endpoint can use q = n + 1 = 2^96, so m_sigma * q needs at most 121 bits and
// still fits in u128. Larger prefixes would need a wider product type.
const NATIVE_UNIFORM_MAX_BITS: u32 = 96;

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
fn shifted_u128_words(value: u128, shift: u32) -> Option<[u64; 4]> {
    if shift >= 256 || shifted_bit_len(value, shift) > 256 {
        return None;
    }

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

    Some(out)
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

    let lhs = shifted_u128_words(lhs, lhs_shift).expect("comparison exceeds fixed scratch");
    let rhs = shifted_u128_words(rhs, rhs_shift).expect("comparison exceeds fixed scratch");

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

enum F32CellCertification {
    Output(f32),
    Unknown,
}

#[derive(Clone, Copy)]
struct Dyadic {
    negative: bool,
    mantissa: u128,
    exponent: i32,
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

// Decode finite IEEE floats exactly as signed dyadics:
//
//     value = (-1)^negative * mantissa * 2^exponent.
//
// The final certificate only manipulates these integer mantissas and exponents.
// Floating-point arithmetic below is allowed to guess candidate cells, but not
// to prove that a cell contains the real affine endpoint.
fn finite_f64_to_dyadic(value: f64) -> Dyadic {
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

    Dyadic {
        negative,
        mantissa,
        exponent,
    }
}

fn finite_f32_to_dyadic(value: f32) -> Dyadic {
    debug_assert!(value.is_finite());

    let bits = value.to_bits();
    let negative = bits >> 31 != 0;
    let raw_exponent = ((bits >> 23) & 0xff) as i32;
    let fraction = bits & ((1u32 << 23) - 1);

    let (mantissa, exponent) = if raw_exponent == 0 {
        (fraction as u128, -149)
    } else {
        (((1u32 << 23) | fraction) as u128, raw_exponent - 127 - 23)
    };

    Dyadic {
        negative,
        mantissa,
        exponent,
    }
}

fn bit_len(value: u128) -> u32 {
    if value == 0 {
        0
    } else {
        u128::BITS - value.leading_zeros()
    }
}

fn term_top_bit(term: &Dyadic) -> Option<i32> {
    let bits = bit_len(term.mantissa);
    if bits == 0 {
        None
    } else {
        Some(term.exponent + bits as i32 - 1)
    }
}

fn add_words(acc: &mut [u64; 4], addend: [u64; 4]) -> Option<()> {
    let mut carry = 0u128;
    for (lhs, rhs) in acc.iter_mut().zip(addend) {
        let sum = *lhs as u128 + rhs as u128 + carry;
        *lhs = sum as u64;
        carry = sum >> 64;
    }
    (carry == 0).then_some(())
}

fn cmp_words(lhs: &[u64; 4], rhs: &[u64; 4]) -> std::cmp::Ordering {
    lhs.iter().rev().cmp(rhs.iter().rev())
}

fn sign_dyadic_sum(terms: &[Dyadic]) -> Option<std::cmp::Ordering> {
    let terms: Vec<_> = terms.iter().copied().filter(|t| t.mantissa != 0).collect();
    if terms.is_empty() {
        return Some(std::cmp::Ordering::Equal);
    }

    // Align all terms to the smallest exponent and accumulate positives and
    // negatives separately in 256 bits. If the aligned span is too wide, this
    // comparison is unresolved unless the dominance check below can decide it.
    let min_exponent = terms.iter().map(|t| t.exponent).min()?;
    let mut positive = [0u64; 4];
    let mut negative = [0u64; 4];

    let mut can_accumulate = true;
    for term in &terms {
        let shift = term.exponent.checked_sub(min_exponent)? as u32;
        let Some(words) = shifted_u128_words(term.mantissa, shift) else {
            can_accumulate = false;
            break;
        };
        if term.negative {
            add_words(&mut negative, words)?;
        } else {
            add_words(&mut positive, words)?;
        }
    }

    if can_accumulate {
        return Some(cmp_words(&positive, &negative));
    }

    // If the highest bit of one term is far above all other terms together,
    // its sign determines the sum without needing a huge common denominator.
    let mut ranked: Vec<_> = terms
        .iter()
        .filter_map(|term| term_top_bit(term).map(|top| (top, term.negative)))
        .collect();
    ranked.sort_by_key(|(top, _)| *top);
    let (top, top_negative) = *ranked.last()?;
    let Some((second, _)) = ranked.iter().rev().nth(1).copied() else {
        return Some(if top_negative {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        });
    };

    // There are at most four terms. If the leading bit is at least four places
    // above the next possible leading bit, the lower terms cannot cancel it.
    if top - second >= 4 {
        Some(if top_negative {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        })
    } else {
        None
    }
}

fn product_term(negative: bool, lhs: u128, rhs: u128, exponent: i32) -> Option<Dyadic> {
    Some(Dyadic {
        negative,
        mantissa: lhs.checked_mul(rhs)?,
        exponent,
    })
}

// Compare the exact real endpoint
//
//     mu +/- scale * (k + q * 2^-bits)
//
// against an exact f32 cell boundary. This is the core proof step. Returning
// None means the fixed native scratch could not certify the sign, so the caller
// must refine the same trace or charge the public comb event at the prefix cap.
fn compare_affine_to_boundary(
    mu: f64,
    scale: f32,
    boundary: f64,
    negative: bool,
    k: u64,
    q: u128,
    bits: u32,
) -> Option<std::cmp::Ordering> {
    debug_assert!(boundary.is_finite());

    let mu = finite_f64_to_dyadic(mu);
    let mut boundary = finite_f64_to_dyadic(boundary);
    boundary.negative = !boundary.negative;

    let scale = finite_f32_to_dyadic(scale);
    let signed_noise_is_negative = negative;
    let scale_k = product_term(
        signed_noise_is_negative,
        scale.mantissa,
        k as u128,
        scale.exponent,
    )?;
    let scale_q = product_term(
        signed_noise_is_negative,
        scale.mantissa,
        q,
        scale.exponent.checked_sub(bits as i32)?,
    )?;

    sign_dyadic_sum(&[mu, boundary, scale_k, scale_q])
}

fn f32_cell_bounds(candidate: f32) -> (Option<f64>, Option<f64>) {
    if candidate == f32::INFINITY {
        let max = f32::MAX as f64;
        let prev = f32::MAX.next_down_() as f64;
        return (Some(max + (max - prev) * 0.5), None);
    }

    if candidate == f32::NEG_INFINITY {
        let min = f32::MIN as f64;
        let next = f32::MIN.next_up_() as f64;
        return (None, Some(min - (next - min) * 0.5));
    }

    debug_assert!(candidate.is_finite());

    if candidate == 0.0 {
        let lower = (candidate.next_down_() as f64) * 0.5;
        let upper = (candidate.next_up_() as f64) * 0.5;
        return (Some(lower), Some(upper));
    }

    let lower = {
        let down = candidate.next_down_();
        if down == f32::NEG_INFINITY {
            let candidate_f64 = candidate as f64;
            let up = candidate.next_up_() as f64;
            candidate_f64 - (up - candidate_f64) * 0.5
        } else {
            ((down as f64) + (candidate as f64)) * 0.5
        }
    };

    let upper = {
        let up = candidate.next_up_();
        if up == f32::INFINITY {
            let candidate_f64 = candidate as f64;
            let down = candidate.next_down_() as f64;
            candidate_f64 + (candidate_f64 - down) * 0.5
        } else {
            ((candidate as f64) + (up as f64)) * 0.5
        }
    };

    (Some(lower), Some(upper))
}

// Native arithmetic is used only to find a nearby cell to try. The authoritative
// decision is the checked dyadic containment test in
// certify_real_affine_rounds_to_f32.
fn candidate_hint(
    mu: f64,
    scale: f32,
    clip: Option<f64>,
    negative: bool,
    k: u64,
    x: &NativeUniform01,
) -> f32 {
    let x_mid = if x.bits == 0 {
        0.5
    } else {
        (x.prefix as f64 + 0.5) * 2.0f64.powi(-(x.bits as i32))
    };
    let signed = if negative { -1.0 } else { 1.0 };
    let value = mu + signed * (scale as f64) * (k as f64 + x_mid);
    let value = clip
        .map(|range| value.max(-range).min(range))
        .unwrap_or(value);
    value as f32
}

fn candidate_set(hint: f32) -> [f32; 3] {
    [hint, hint.next_down_(), hint.next_up_()]
}

fn compare_clipped_endpoint_to_boundary(
    mu: f64,
    scale: f32,
    clip: Option<f64>,
    negative: bool,
    k: u64,
    q: u128,
    bits: u32,
    boundary: f64,
) -> Option<std::cmp::Ordering> {
    let Some(range) = clip else {
        return compare_affine_to_boundary(mu, scale, boundary, negative, k, q, bits);
    };

    // Clipping is monotone. First decide whether the unclipped endpoint is
    // below -range or above +range; if so, compare that constant clipped value
    // to the cell boundary. Otherwise compare the original affine endpoint.
    let lower_cmp = compare_affine_to_boundary(mu, scale, -range, negative, k, q, bits)?;
    if lower_cmp.is_lt() {
        return compare_affine_to_boundary(-range, 1.0, boundary, false, 0, 0, 0);
    }

    let upper_cmp = compare_affine_to_boundary(mu, scale, range, negative, k, q, bits)?;
    if upper_cmp.is_gt() {
        return compare_affine_to_boundary(range, 1.0, boundary, false, 0, 0, 0);
    }

    compare_affine_to_boundary(mu, scale, boundary, negative, k, q, bits)
}

fn certify_real_affine_rounds_to_f32(
    mu: f64,
    scale: f32,
    clip: Option<f64>,
    negative: bool,
    k: u64,
    x: &NativeUniform01,
) -> F32CellCertification {
    // For S = +1, the lower endpoint uses q = n and the upper endpoint uses
    // q = n + 1. For S = -1, the affine map reverses the prefix interval.
    let lo_q = if negative { x.prefix + 1 } else { x.prefix };
    let hi_q = if negative { x.prefix } else { x.prefix + 1 };

    for candidate in candidate_set(candidate_hint(mu, scale, clip, negative, k, x)) {
        let (lower, upper) = f32_cell_bounds(candidate);
        let lower_inside = lower.is_none_or(|boundary| {
            compare_clipped_endpoint_to_boundary(
                mu, scale, clip, negative, k, lo_q, x.bits, boundary,
            )
            .is_some_and(|ordering| ordering.is_ge())
        });
        let upper_inside = upper.is_none_or(|boundary| {
            compare_clipped_endpoint_to_boundary(
                mu, scale, clip, negative, k, hi_q, x.bits, boundary,
            )
            .is_some_and(|ordering| ordering.is_le())
        });

        if lower_inside && upper_inside {
            return F32CellCertification::Output(candidate);
        }
    }

    // Unknown is not a sampler rejection by itself. The caller refines the same
    // accepted trace until the prefix cap; only then is it the declared comb
    // rejection event used in the privacy accounting.
    F32CellCertification::Unknown
}

/// Sample with f64 input parameters and try to certify the exact real affine
/// result directly into an extended f32 output cell.
///
/// This does not construct a floating-point noise value and then add it to
/// `mu`. The sampled Karney variate remains a lazy real interval, and this
/// routine returns only after proving that the real value
///
///     mu +/- scale32 * (k + x)
///
/// rounds to one f32 cell, including overflow cells represented by infinities.
/// `scale32` is the smallest finite positive f32 at least as large as the
/// requested scale.
///
/// Failure to certify before the native PSRN cap rejects the accepted trace as
/// an unresolved rounding-boundary comb. This slightly conditions the target
/// law and is charged by the comb normalization term. K remains capped by u64;
/// its half-Gaussian tail is a separate sampler-side finite-budget term and is
/// intentionally not traded down unless accounting shows it is the bottleneck.
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
    let scale = snap_scale_up_to_f32(scale)?;

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
