//! Certified rejection sampling of a rounded Bingham vector.
//!
//! Approximate linear algebra only supplies a proposal. All decisions and
//! output rounding are certified against the exact (dyadic) target matrix.
use dashu::{float::{Context, round::mode::{Down, Up}}, integer::{IBig, UBig, ops::SquareRoot}, rational::RBig};
use crate::{error::Fallible, traits::{CastInternalRational, RoundCast}};
use super::{PartialSample, karney::sample_exact_std_normal as normal, psrn::{PartialUniform01, Uniform01RV}};

#[derive(Clone, Debug)]
struct Interval {
    lo: RBig,
    hi: RBig,
}

impl Interval {
    fn point(x: RBig) -> Self {
        Self { lo: x.clone(), hi: x }
    }
    fn add(&self, rhs: &Self) -> Self {
        Self { lo: &self.lo + &rhs.lo, hi: &self.hi + &rhs.hi }
    }
    fn sub(&self, rhs: &Self) -> Self {
        Self { lo: &self.lo - &rhs.hi, hi: &self.hi - &rhs.lo }
    }
    fn mul(&self, rhs: &Self) -> Self {
        let products = [
            &self.lo * &rhs.lo, &self.lo * &rhs.hi,
            &self.hi * &rhs.lo, &self.hi * &rhs.hi,
        ];
        Self {
            lo: products.iter().min().unwrap().clone(),
            hi: products.iter().max().unwrap().clone(),
        }
    }
    fn scale(&self, c: &RBig) -> Self {
        if c < &RBig::ZERO {
            Self { lo: &self.hi * c, hi: &self.lo * c }
        } else {
            Self { lo: &self.lo * c, hi: &self.hi * c }
        }
    }
    fn div_positive(&self, rhs: &Self) -> Option<Self> {
        if rhs.lo <= RBig::ZERO {
            return None;
        }
        Some(self.mul(&Self {
            lo: RBig::ONE / &rhs.hi,
            hi: RBig::ONE / &rhs.lo,
        }))
    }
    fn square(&self) -> Self {
        let a = self.lo.pow(2);
        let b = self.hi.pow(2);
        Self {
            lo: if self.lo <= RBig::ZERO && self.hi >= RBig::ZERO {
                RBig::ZERO
            } else {
                a.clone().min(b.clone())
            },
            hi: a.max(b),
        }
    }

}

fn one() -> Interval {
    Interval::point(RBig::ONE)
}

fn zero() -> Interval {
    Interval::point(RBig::ZERO)
}

// Both rational conversion and ln are correctly rounded in the chosen
// direction (Dashu >= 0.6). Monotonicity gives a convergent enclosure.
fn log_interval(x: &Interval, bits: usize) -> Fallible<Option<Interval>> {
    if x.lo <= RBig::ZERO {
        return Ok(None);
    }
    let lo = x.lo.to_float::<Down, 2>(bits).value();
    let hi = x.hi.to_float::<Up, 2>(bits).value();
    let lo = Context::<Down>::new(bits).ln(lo.repr(), None)
        .map_err(|e| err!(FailedFunction, "Bingham logarithm: {:?}", e))?.value();
    let hi = Context::<Up>::new(bits).ln(hi.repr(), None)
        .map_err(|e| err!(FailedFunction, "Bingham logarithm: {:?}", e))?.value();
    Ok(Some(Interval {
        lo: RBig::try_from(lo).map_err(|_| err!(FailedFunction, "nonfinite logarithm"))?,
        hi: RBig::try_from(hi).map_err(|_| err!(FailedFunction, "nonfinite logarithm"))?,
    }))
}

fn uniform() -> Fallible<PartialUniform01> {
    let mut sample = PartialSample::new(Uniform01RV);
    sample.refine()?;
    Ok(sample)
}

fn bounds(sample: &PartialUniform01) -> Interval {
    Interval { lo: sample.lower().unwrap(), hi: sample.upper().unwrap() }
}

fn identity(d: usize) -> Vec<Vec<RBig>> {
    (0..d).map(|i| (0..d).map(|j| RBig::from(u8::from(i == j))).collect()).collect()
}
fn transpose<T: Clone>(a: &[Vec<T>]) -> Vec<Vec<T>> {
    (0..a.len()).map(|j| a.iter().map(|row| row[j].clone()).collect()).collect()
}

// All entries of P and B are dyadic, so the largest denominator is
// divisible by every other denominator. Lift the matrices to integers
// once instead of normalizing a rational after each multiply and add.
fn integer_matrix(a: &[Vec<RBig>]) -> (Vec<Vec<IBig>>, UBig) {
    let denominator = a.iter().flatten()
        .map(|x| x.clone().into_parts().1)
        .max().unwrap();
    let numerators = a.iter().map(|row| row.iter().map(|x| {
        let (n, d) = x.clone().into_parts();
        n * IBig::from(&denominator / d)
    }).collect()).collect();
    (numerators, denominator)
}

fn matmul(a: &[Vec<IBig>], b: &[Vec<IBig>]) -> Vec<Vec<IBig>> {
    let bt = transpose(b);
    a.iter().map(|row| bt.iter().map(|col| {
        row.iter().zip(col).fold(IBig::ZERO, |acc, (l, r)| acc + l * r)
    }).collect()).collect()
}

fn rationalize(a: Vec<Vec<IBig>>, denominator: &UBig) -> Vec<Vec<RBig>> {
    a.into_iter().map(|row| row.into_iter().map(|n| {
        RBig::from_parts(n, denominator.clone())
    }).collect()).collect()
}
// Entrywise enclosure: |exact[i,j] - values[i,j]| <= error.
// The magnitude is the largest absolute *floating-point* entry,
// represented as an exact rational.
struct ApproxMatrix {
    values: Vec<Vec<f64>>,
    error: RBig,
    magnitude: RBig,
}

impl ApproxMatrix {
    fn new(values: Vec<Vec<f64>>, error: RBig) -> Option<Self> {
        if values.iter().flatten().any(|x| !x.is_finite()) {
            return None;
        }
        let magnitude = values.iter().flatten().fold(0.0f64, |m, x| m.max(x.abs()))
            .into_rational().ok()?;
        Some(Self { values, error, magnitude })
    }

    fn from_exact(matrix: &[Vec<RBig>]) -> Option<Self> {
        let mut error = RBig::ZERO;
        let mut values = Vec::with_capacity(matrix.len());
        for row in matrix {
            let mut rounded = Vec::with_capacity(row.len());
            for x in row {
                let f = f64::round_cast(x.clone()).ok()?;
                error = error.max(abs(&(x - f.into_rational().ok()?)));
                rounded.push(f);
            }
            values.push(rounded);
        }
        Self::new(values, error)
    }

    fn transpose(&self) -> Self {
        Self {
            values: transpose(&self.values),
            error: self.error.clone(),
            magnitude: self.magnitude.clone(),
        }
    }

    fn multiply(&self, rhs: &Self) -> Option<Self> {
        let d = RBig::from(self.values.len());
        let eps = f64::EPSILON.into_rational().ok()?;
        let tiny = f64::from_bits(1).into_rational().ok()?;
        // At most 2d roundings per dot product. With 2d*eps <= 1/2,
        // (1+eps)^(2d)-1 <= 4d*eps, and propagated additive
        // underflow errors total at most 4d*tiny.
        let gamma = RBig::from(4) * &d * eps;
        if gamma > RBig::ONE {
            return None;
        }
        // Error from rounding the inputs, plus error of the dot product.
        let input_error = &d * (
            &self.error * (&rhs.magnitude + &rhs.error)
            + &self.magnitude * &rhs.error
        );
        let arithmetic_error = gamma * &d * &self.magnitude * &rhs.magnitude
            + RBig::from(4) * &d * tiny;
        let rt = transpose(&rhs.values);
        let values = self.values.iter().map(|row| rt.iter().map(|col| {
            row.iter().zip(col).fold(0.0, |acc, (l, r)| acc + l * r)
        }).collect()).collect();
        Self::new(values, input_error + arithmetic_error)
    }

    // For a symmetric exact matrix A, the error in any entry of the
    // approximation is at most error. Inflate each Gershgorin radius
    // by d*error (including error in the diagonal).
    fn diagonal_bounds(&self) -> Option<Vec<Interval>> {
        let inflate = RBig::from(self.values.len()) * &self.error;
        self.values.iter().enumerate().map(|(i, row)| {
            let diag = row[i].into_rational().ok()?;
            let radius = row.iter().enumerate().filter(|(j, _)| *j != i)
                .try_fold(inflate.clone(), |acc, (_, x)| {
                    Some(acc + x.abs().into_rational().ok()?)
                })?;
            Some(Interval { lo: &diag - &radius, hi: &diag + &radius })
        }).collect()
    }
}

fn approximate_diagonals(p: &[Vec<RBig>], factor: &[Vec<RBig>]) -> Option<(Vec<Interval>, Vec<Interval>)> {
    let p = ApproxMatrix::from_exact(p)?;
    let b = ApproxMatrix::from_exact(factor)?;
    let bt = b.transpose();
    let g = bt.multiply(&p.multiply(&b)?)?.diagonal_bounds()?;
    let h = bt.multiply(&b)?.diagonal_bounds()?;
    Some((g, h))
}

fn exact_grams(p: &[Vec<RBig>], factor: &[Vec<RBig>]) -> (Vec<Vec<RBig>>, Vec<Vec<RBig>>) {
    let (p, p_den) = integer_matrix(p);
    let (b, b_den) = integer_matrix(factor);
    let bt = transpose(&b);
    let h_den = &b_den * &b_den;
    let g = rationalize(matmul(&bt, &matmul(&p, &b)), &(&p_den * &h_den));
    let h = rationalize(matmul(&bt, &b), &h_den);
    (g, h)
}

fn abs(x: &RBig) -> RBig {
    if x < &RBig::ZERO { -x } else { x.clone() }
}

// Gershgorin bounds on each row, also usable as diagonal quadratic-form bounds.
fn diagonal_bounds(a: &[Vec<RBig>]) -> Vec<Interval> {
    a.iter().enumerate().map(|(i, row)| {
        let radius = row.iter().enumerate().filter(|(j, _)| *j != i)
            .fold(RBig::ZERO, |acc, (_, x)| acc + abs(x));
        Interval { lo: &row[i] - &radius, hi: &row[i] + &radius }
    }).collect()
}

fn linear(row: &[RBig], z: &[Interval]) -> Interval {
    row.iter().zip(z).fold(zero(), |acc, (a, x)| acc.add(&x.scale(a)))
}
fn quadratic(a: &[Vec<RBig>], z: &[Interval]) -> Interval {
    a.iter().zip(z).fold(zero(), |acc, (row, x)| acc.add(&linear(row, z).mul(x)))
}
fn diagonal_form(a: &[Interval], z2: &[Interval]) -> Interval {
    a.iter().zip(z2).fold(zero(), |acc, (a, x)| acc.add(&a.mul(x)))
}
fn norm_squared(z: &[Interval]) -> Interval {
    z.iter().fold(zero(), |acc, x| acc.add(&x.square()))
}

struct Proposal {
    factor: Vec<Vec<RBig>>,
    precision: Vec<Vec<RBig>>,
    g_diagonal: Vec<Interval>,
    h_diagonal: Vec<Interval>,
    lower: RBig,
}

impl Proposal {
    fn new(p: Vec<Vec<RBig>>, factor: Vec<Vec<RBig>>) -> Option<Self> {
        let certified = approximate_diagonals(&p, &factor)
            .filter(|(g, _)| g.iter().all(|r| r.lo > RBig::ZERO));
        let (g_diagonal, h_diagonal) = certified.unwrap_or_else(|| {
            let (g, h) = exact_grams(&p, &factor);
            (diagonal_bounds(&g), diagonal_bounds(&h))
        });
        Self::from_bounds(p, factor, g_diagonal, h_diagonal)
    }

    fn from_bounds(p: Vec<Vec<RBig>>, factor: Vec<Vec<RBig>>, g_diagonal: Vec<Interval>, h_diagonal: Vec<Interval>) -> Option<Self> {
        let lower = g_diagonal.iter().map(|r| &r.lo).min()?.clone();
        if lower <= RBig::ZERO {
            return None;
        }
        // G >= lower I proves that both P and the actual proposal are nonsingular.
        Some(Self { factor, precision: p, g_diagonal, h_diagonal, lower })
    }

    fn decide(
        &self, g: &Interval, h: &Interval, z2: &Interval,
        log_u: &Interval, bits: usize,
    ) -> Fallible<Option<bool>> {
        let d = RBig::from(self.factor.len());
        let denominator = h.scale(&d);
        let Some(x) = g.div_positive(&denominator) else { return Ok(None); };
        let Some(y) = z2.scale(&self.lower).div_positive(&denominator) else { return Ok(None); };
        let Some(log_y) = log_interval(&y, bits)? else { return Ok(None); };
        let rhs = log_y.add(&one()).sub(&x).scale(&d);
        let lhs = log_u.scale(&RBig::from(2));
        Ok(if lhs.hi < rhs.lo {
            Some(true)
        } else if lhs.lo > rhs.hi {
            Some(false)
        } else {
            None
        })
    }

    fn sample(&self) -> Fallible<Vec<f64>> {
        loop {
            let mut z = (0..self.factor.len()).map(|_| normal()).collect::<Fallible<Vec<_>>>()?;
            let mut u = uniform()?;
            let mut bits = 64;
            let mut accepted = false;
            loop {
                {
                    let z_bounds: Vec<_> = z.iter().map(|x| {
                        let (lo, hi) = x.bounds();
                        Interval { lo, hi }
                    }).collect();
                    let mut v = None;
                    let project = || self.factor.iter().map(|row| linear(row, &z_bounds)).collect::<Vec<_>>();
                    if !accepted {
                        if let Some(log_u) = log_interval(&bounds(&u), bits)? {
                            let squared: Vec<_> = z_bounds.iter().map(Interval::square).collect();
                            let z2 = squared.iter().fold(zero(), |acc, x| acc.add(x));
                            let g = diagonal_form(&self.g_diagonal, &squared);
                            let h = diagonal_form(&self.h_diagonal, &squared);
                            let decision = match self.decide(&g, &h, &z2, &log_u, bits)? {
                                None => {
                                    let v = v.insert(project());
                                    self.decide(&quadratic(&self.precision, v), &norm_squared(v), &z2, &log_u, bits)?
                                }
                                decision => decision,
                            };
                            match decision {
                                Some(true) => accepted = true,
                                Some(false) => break,
                                None => (),
                            }
                        }
                    }
                    if accepted {
                        if let Some(result) = round_vector(&v.unwrap_or_else(project), bits)? {
                            return Ok(result);
                        }
                    }
                }
                for x in &mut z { x.refine()?; }
                if !accepted { u.refine()?; }
                bits += 64;
            }
        }
    }
}

// Enclose sqrt(q) on a dyadic grid using the exact integer square root:
// floor(2^bits sqrt(q)) = floor(sqrt(floor(2^(2bits) q))).
fn sqrt_bounds(q: &RBig, bits: usize) -> Interval {
    let unit = RBig::from(IBig::ONE << bits);
    let n = (q * &unit * &unit).floor().sqrt();
    let lo = RBig::from(n) / &unit;
    let hi = if lo.pow(2) == *q { lo.clone() } else { &lo + RBig::ONE / unit };
    Interval { lo, hi }
}

fn round_vector(v: &[Interval], bits: usize) -> Fallible<Option<Vec<f64>>> {
    let norm2 = norm_squared(v);
    let norm = Interval {
        lo: sqrt_bounds(&norm2.lo, bits).lo,
        hi: sqrt_bounds(&norm2.hi, bits).hi,
    };
    let Some(reciprocal) = one().div_positive(&norm) else {
        return Ok(None);
    };
    v.iter().map(|x| {
        let ratio = x.mul(&reciprocal);
        // Round magnitudes and restore the sign explicitly, including -0.
        let round = |r: RBig| -> Fallible<f64> {
            if r < RBig::ZERO { Ok(-f64::round_cast(-r)?) } else { f64::round_cast(r) }
        };
        let lo = round(ratio.lo)?;
        let hi = round(ratio.hi)?;
        Ok((lo.to_bits() == hi.to_bits()).then_some(lo))
    }).collect::<Fallible<Vec<_>>>().map(|v| v.into_iter().collect())
}

fn rational_matrix(matrix: &[f64], d: usize) -> Fallible<Vec<Vec<RBig>>> {
    matrix.chunks_exact(d).map(|row| row.iter().map(|x| x.into_rational()).collect()).collect()
}

fn precision(c: &[Vec<RBig>], k: &RBig, b: &RBig, l: &RBig) -> Vec<Vec<RBig>> {
    let two_k = k * RBig::from(2);
    c.iter().enumerate().map(|(i, row)| row.iter().enumerate().map(|(j, x)| {
        if i == j { b + &two_k * (l - x) } else { -&two_k * x }
    }).collect()).collect()
}

/// Sample the Bingham density proportional to exp(concentration * u^T C u)
/// with respect to uniform sphere measure, then round each coordinate to f64.
/// Matrix and factor are row-major; factor, b and l are untrusted numerical
/// hints. The latter is relative to the first diagonal entry of C.
/// A failed certificate falls back to exact (potentially slow) uniform-sphere
/// rejection. Runtime is not constant and must not be exposed as a release.
pub(crate) fn sample_bingham(
    matrix: &[f64], dimension: usize, concentration: f64,
    factor: &[f64], b: f64, l: f64,
) -> Fallible<Vec<f64>> {
    let d = dimension;
    if d == 0 || d.checked_mul(d) != Some(matrix.len()) || !concentration.is_finite() || concentration < 0.0 {
        return fallible!(FailedFunction, "invalid Bingham matrix shape or concentration");
    }
    let mut c = rational_matrix(matrix, d)?;
    if d == 1 {
        return Ok(vec![if super::sample_standard_bernoulli()? { 1.0 } else { -1.0 }]);
    }
    let shift = c[0][0].clone();
    // A quadratic form only depends on the symmetric part. Keep the
    // translation exact, even if the numerical proposal used a rounded shift.
    for i in 0..d {
        c[i][i] -= &shift;
        for j in 0..i {
            let v = (&c[i][j] + &c[j][i]) / RBig::from(2);
            c[i][j] = v.clone();
            c[j][i] = v;
        }
    }
    let k = concentration.into_rational()?;
    if factor.len() == matrix.len() && factor.iter().all(|x| x.is_finite()) && b.is_finite() && l.is_finite() {
        let p = precision(&c, &k, &b.into_rational()?, &l.into_rational()?);
        if let Some(proposal) = Proposal::new(p, rational_matrix(factor, d)?) {
            return proposal.sample();
        }
    }
    // Gershgorin provides an exact upper bound on the shifted matrix's
    // largest eigenvalue. With B=I and b=d the acceptance rule reduces to
    // ordinary uniform-sphere rejection with this bound.
    let upper = diagonal_bounds(&c).into_iter().map(|r| r.hi).max().unwrap();
    let p = precision(&c, &k, &RBig::from(d), &upper);
    // G = P is diagonally dominant with lower bound at least d > 0.
    let g_diagonal = diagonal_bounds(&p);
    Proposal::from_bounds(p, identity(d), g_diagonal, vec![one(); d]).unwrap().sample()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directed_log_bounds_include_inexact_rational_inputs() -> Fallible<()> {
        // Use a rigorous independent bound for ln(2): 2*atanh(1/3).
        let t = RBig::ONE / RBig::from(3);
        let partial = (0..24).fold(RBig::ZERO, |sum, j| {
            sum + t.pow(2 * j + 1) / RBig::from(2 * j + 1)
        }) * RBig::from(2);
        let upper = &partial + t.pow(49) * RBig::from(3);
        let bounds = log_interval(&Interval::point(RBig::from(2)), 16)?.unwrap();
        assert!(bounds.lo <= partial && upper <= bounds.hi);
        let exact = log_interval(&one(), 16)?.unwrap();
        assert_eq!(exact.lo, RBig::ZERO);
        assert_eq!(exact.hi, RBig::ZERO);
        let third = log_interval(&Interval::point(RBig::ONE / RBig::from(3)), 16)?.unwrap();
        let three = log_interval(&Interval::point(RBig::from(3)), 16)?.unwrap();
        assert!(third.lo <= -&three.lo && -&three.hi <= third.hi);
        assert!(log_interval(&zero(), 16)?.is_none());
        Ok(())
    }

    #[test]
    fn square_root_bounds_and_rounding_include_subnormals_and_ties() -> Fallible<()> {
        for x in [0.0, f64::from_bits(1), f64::MIN_POSITIVE, 0.5, 1.0] {
            let q = x.into_rational()?.pow(2);
            let interval = sqrt_bounds(&q, 1075);
            assert_eq!(interval.lo, interval.hi);
            assert_eq!(f64::round_cast(interval.lo)?.to_bits(), x.to_bits());
        }
        let midpoint = (0.5f64.into_rational()? + f64::from_bits(0.5f64.to_bits() + 1).into_rational()?) / RBig::from(2);
        let interval = sqrt_bounds(&midpoint.pow(2), 64);
        assert_eq!(interval.lo, interval.hi);
        assert_eq!(f64::round_cast(interval.lo)?, 0.5);
        let q = RBig::from(2) / RBig::from(3);
        let bounds = sqrt_bounds(&q, 64);
        assert!(bounds.lo.pow(2) <= q && q <= bounds.hi.pow(2));
        assert_eq!(bounds.hi - bounds.lo, RBig::ONE / RBig::from(IBig::ONE << 64));
        Ok(())
    }

    #[test]
    fn shared_normalization_preserves_signed_zero() -> Fallible<()> {
        let tiny = RBig::from(IBig::ONE << 1076);
        let v = [Interval::point(-RBig::ONE / &tiny), one()];
        let rounded = round_vector(&v, 1080)?.unwrap();
        assert_eq!(rounded[0].to_bits(), (-0.0f64).to_bits());
        assert_eq!(rounded[1], 1.0);
        let v = [Interval { lo: -RBig::ONE / &tiny, hi: RBig::ONE / &tiny }, one()];
        assert!(round_vector(&v, 1080)?.is_none());
        Ok(())
    }

    #[test]
    fn certificate_rejects_singular_proposal_and_fallback_samples() -> Fallible<()> {
        let p = identity(2);
        assert!(Proposal::new(p, vec![vec![RBig::ZERO; 2]; 2]).is_none());
        let vector = sample_bingham(&[1.0, 0.0, 0.0, 0.0], 2, 1.0, &[], 1.0, 0.0)?;
        assert!((vector.iter().map(|x| x*x).sum::<f64>() - 1.0).abs() < 1e-14);
        Ok(())
    }

    #[test]
    fn integer_gram_matrices_preserve_dyadic_entries() -> Fallible<()> {
        let factor = rational_matrix(&[0.5, -0.125, 0.25, 1.0], 2)?;
        let p = rational_matrix(&[2.0, 0.5, 0.5, 1.0], 2)?;
        let (g, h) = exact_grams(&p, &factor);
        // B^T P B and B^T B, including off-diagonal cancellation.
        assert_eq!(g, rational_matrix(&[0.6875, 0.359375, 0.359375, 0.90625], 2)?);
        assert_eq!(h, rational_matrix(&[0.3125, 0.1875, 0.1875, 1.015625], 2)?);
        let (g_bounds, h_bounds) = approximate_diagonals(&p, &factor).unwrap();
        for (bounds, exact) in [(g_bounds, g), (h_bounds, h)] {
            for (bound, exact_bound) in bounds.iter().zip(diagonal_bounds(&exact)) {
                assert!(bound.lo <= exact_bound.lo && exact_bound.hi <= bound.hi);
            }
        }
        Ok(())
    }

    #[test]
    fn approximate_product_accounts_for_input_rounding_and_underflow() -> Fallible<()> {
        let third = RBig::ONE / RBig::from(3);
        let tiny = f64::from_bits(1).into_rational()?;
        let a = vec![vec![third.clone(), tiny.clone()], vec![-&tiny, third]];
        let b = vec![vec![RBig::from(2), tiny.clone()], vec![tiny, -RBig::ONE]];
        let approximate = ApproxMatrix::from_exact(&a).unwrap()
            .multiply(&ApproxMatrix::from_exact(&b).unwrap()).unwrap();
        for i in 0..2 {
            for j in 0..2 {
                let exact = &a[i][0] * &b[0][j] + &a[i][1] * &b[1][j];
                assert!(abs(&(exact - approximate.values[i][j].into_rational()?)) <= approximate.error);
            }
        }
        let huge = vec![vec![f64::MAX.into_rational()?]];
        let huge = ApproxMatrix::from_exact(&huge).unwrap();
        assert!(huge.multiply(&huge).is_none());
        Ok(())
    }

    #[test]
    fn diagonal_screen_encloses_dense_form() {
        let a = vec![vec![RBig::from(2), RBig::ONE], vec![RBig::ONE, RBig::from(3)]];
        let z = vec![Interval::point(RBig::from(2)), Interval::point(RBig::from(-3))];
        let exact = quadratic(&a, &z);
        let screened = diagonal_form(&diagonal_bounds(&a), &z.iter().map(Interval::square).collect::<Vec<_>>());
        assert!(screened.lo <= exact.lo && exact.hi <= screened.hi);
    }
}
