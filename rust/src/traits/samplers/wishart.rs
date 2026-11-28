//! Correctly rounded difference-of-Wisharts sampling.
//!
//! For dimension d, draw independent d x (d+1) standard normal matrices U,V.
//! Return the upper triangle of G + q (UV^T + VU^T). This has the law of
//! G + q (AA^T - BB^T) for independent standard normal A,B of the same shape.
//! With q > 0 the exact mechanism is 1/(2q)-Lipschitz private in nuclear distance.
//! Runtime is not covered by this output-distribution guarantee.

use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
};

use super::karney::{ExactStdNormal, sample_exact_std_normal};
use crate::{error::Fallible, traits::CastInternalRational};

struct GridRow {
    values: Vec<IBig>,
    norm: IBig,
}

impl GridRow {
    fn new(values: Vec<IBig>) -> Self {
        let norm = values.iter().fold(
            IBig::ZERO,
            |sum, x| {
                if x < &IBig::ZERO { sum - x } else { sum + x }
            },
        );
        Self { values, norm }
    }
}

// Each sampled coordinate lies in [a/S, (a+1)/S] for its grid integer a.
// Therefore the error of a product ab in units of S^-2 is at most
// |a|+|b|+1. Sum this bound over both dot products, using cached row norms.
fn dot_bounds(a: &GridRow, b: &GridRow) -> (IBig, IBig) {
    let m = a.values.len() / 2;
    let product = (0..m).fold(IBig::ZERO, |sum, k| {
        sum + &a.values[k] * &b.values[m + k] + &a.values[m + k] * &b.values[k]
    });
    let error = &a.norm + &b.norm + IBig::from(2 * m);
    (&product - &error, product + error)
}

fn round_interval(lo: RBig, hi: RBig) -> Option<f64> {
    // Keep the sign of a negative value rounded to zero.
    let round = |x: RBig| {
        if x < RBig::ZERO {
            -f64::from_rational(-x)
        } else {
            f64::from_rational(x)
        }
    };
    let lo = round(lo);
    let hi = round(hi);
    (lo.to_bits() == hi.to_bits()).then_some(lo)
}

fn sample_factors(d: usize) -> Fallible<Vec<Vec<ExactStdNormal>>> {
    (0..d)
        .map(|_| {
            (0..2 * (d + 1))
                .map(|_| sample_exact_std_normal())
                .collect()
        })
        .collect()
}

fn round_wishart(
    input: &[RBig],
    q: &RBig,
    factors: &mut [Vec<ExactStdNormal>],
) -> Fallible<Vec<f64>> {
    let d = factors.len();
    let mut output = vec![None; input.len()];
    let mut remaining = input.len();
    let mut bits = 64usize;
    while remaining != 0 {
        let grid = RBig::from(IBig::ONE << bits);
        let denom = UBig::ONE << (2 * bits);
        let rows = factors
            .iter_mut()
            .map(|row| {
                let values = row
                    .iter_mut()
                    .map(|z| z.grid_floor(bits, &grid))
                    .collect::<Fallible<Vec<_>>>()?;
                Ok(GridRow::new(values))
            })
            .collect::<Fallible<Vec<_>>>()?;
        let mut index = 0;
        for i in 0..d {
            for j in i..d {
                if output[index].is_none() {
                    let (lo, hi) = dot_bounds(&rows[i], &rows[j]);
                    // Add the private input BEFORE rounding the perturbation.
                    let lo = &input[index] + q * RBig::from_parts(lo, denom.clone());
                    let hi = &input[index] + q * RBig::from_parts(hi, denom.clone());
                    if let Some(value) = round_interval(lo, hi) {
                        output[index] = Some(value);
                        remaining -= 1;
                    }
                }
                index += 1;
            }
        }
        bits += 64;
    }
    Ok(output.into_iter().map(Option::unwrap).collect())
}

/// Sample a correctly rounded Wishart difference added to a symmetric matrix
/// represented by its row-major packed upper triangle. The public scale q
/// must be finite and positive. The caller is responsible for its calibration.
pub(crate) fn sample_wishart_difference(
    upper: &[f64],
    dimension: usize,
    scale: f64,
) -> Fallible<Vec<f64>> {
    if dimension == 0
        || dimension
            .checked_add(1)
            .and_then(|n| dimension.checked_mul(n))
            .map(|n| n / 2)
            != Some(upper.len())
        || !scale.is_finite()
        || scale <= 0.0
    {
        return fallible!(
            FailedFunction,
            "expected packed nonempty square matrix and positive finite scale"
        );
    }
    let input = upper
        .iter()
        .map(|x| x.into_rational())
        .collect::<Fallible<Vec<_>>>()?;
    let q = scale.into_rational()?;
    round_wishart(&input, &q, &mut sample_factors(dimension)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_dot_bounds_enclose_all_cell_corners() {
        let a = [-3, 1, 2, -4];
        let b = [5, -2, -1, 3];
        let (lo, hi) = dot_bounds(
            &GridRow::new(a.into_iter().map(IBig::from).collect()),
            &GridRow::new(b.into_iter().map(IBig::from).collect()),
        );
        for mask in 0..256 {
            let x = std::array::from_fn::<_, 4, _>(|i| a[i] + ((mask >> i) & 1));
            let y = std::array::from_fn::<_, 4, _>(|i| b[i] + ((mask >> (i + 4)) & 1));
            let value = IBig::from(x[0] * y[2] + x[1] * y[3] + x[2] * y[0] + x[3] * y[1]);
            assert!(lo <= value && value <= hi);
        }
    }

    #[test]
    fn rounding_distinguishes_signed_zero_and_midpoints() -> Fallible<()> {
        let tiny = RBig::ONE / RBig::from(IBig::ONE << 1076usize);
        assert_eq!(
            round_interval(-&tiny, -&tiny).unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
        assert!(round_interval(-&tiny, tiny).is_none());
        let a = 1.0f64;
        let b = f64::from_bits(a.to_bits() + 1);
        let midpoint = (a.into_rational()? + b.into_rational()?) / RBig::from(2);
        assert_eq!(round_interval(midpoint.clone(), midpoint), Some(a));
        Ok(())
    }

    #[test]
    fn packed_sampler_accepts_extreme_scales() -> Fallible<()> {
        for q in [f64::from_bits(1), 1.0, f64::MAX] {
            let result = sample_wishart_difference(&[1.0, -2.0, 3.0], 2, q)?;
            assert_eq!(result.len(), 3);
            assert!(result.iter().all(|v| !v.is_nan()));
        }
        assert!(sample_wishart_difference(&[0.0], 2, 1.0).is_err());
        assert!(sample_wishart_difference(&[f64::NAN], 1, 1.0).is_err());
        assert!(sample_wishart_difference(&[0.0], 1, 0.0).is_err());
        Ok(())
    }

    #[test]
    #[ignore = "performance benchmark"]
    fn benchmark_wishart() -> Fallible<()> {
        for d in [8, 32, 128, 256, 512, 1024] {
            let start = std::time::Instant::now();
            let mut factors = sample_factors(d)?;
            let sampling = start.elapsed();
            let input = vec![RBig::ZERO; d * (d + 1) / 2];
            let start = std::time::Instant::now();
            let output = round_wishart(&input, &RBig::ONE, &mut factors)?;
            let arithmetic = start.elapsed();
            std::hint::black_box(output);
            println!(
                "BENCH d={d} normal_ms={:.3} arithmetic_ms={:.3}",
                sampling.as_secs_f64() * 1000.0,
                arithmetic.as_secs_f64() * 1000.0
            );
        }
        Ok(())
    }
}
