use crate::{
    error::Fallible,
    measures::curves::{check_alpha, check_epsilon, check_mu},
    traits::{DirectedScalar, Direction, SInterval, SoftFloat},
};
use errorfunctions::RealErrorFunctions;
use statrs::function::erf::{erfc as erfc_f64, erfc_inv as erfc_inv_f64};
use std::f64::consts::SQRT_2;

type I = SInterval;
type GaussianEndpoint = SoftFloat;

// Special functions remain idealized native approximations, not certified
// transcendental bounds. Preserve the former best-effort endpoint behavior
// locally: widen finite approximations by one ULP, retain infinities, reject
// NaN. Other interval arithmetic uses the existing software backend.
fn approximate_endpoint(value: f64, direction: Direction) -> Fallible<GaussianEndpoint> {
    if value.is_nan() {
        return fallible!(NumericError, "Gaussian special function returned NaN");
    }
    GaussianEndpoint::exact(if value.is_infinite() {
        value
    } else {
        match direction {
            Direction::Down => value.next_down(),
            Direction::Up => value.next_up(),
        }
    })
}

fn erfc_round(value: GaussianEndpoint, direction: Direction) -> Fallible<GaussianEndpoint> {
    // These functions are decreasing: an output lower bound needs an
    // upward-rounded input, and an output upper bound a downward-rounded input.
    let value = value.to_f64(match direction {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
    })?;
    let value = if value == f64::INFINITY {
        0.0
    } else if value == f64::NEG_INFINITY {
        2.0
    } else {
        erfc_f64(value).clamp(0.0, 2.0)
    };
    approximate_endpoint(value, direction)
}

fn erfcx_round(value: GaussianEndpoint, direction: Direction) -> Fallible<GaussianEndpoint> {
    let value = value
        .to_f64(match direction {
            Direction::Down => Direction::Up,
            Direction::Up => Direction::Down,
        })?
        .erfcx();
    if value.is_nan() || value < 0.0 {
        return fallible!(NumericError, "erfcx returned invalid value {value}");
    }
    approximate_endpoint(value, direction)
}

fn erfc_inv_round(value: GaussianEndpoint, direction: Direction) -> Fallible<GaussianEndpoint> {
    // These functions are decreasing: an output lower bound needs an
    // upward-rounded input, and an output upper bound a downward-rounded input.
    let value = value.to_f64(match direction {
        Direction::Down => Direction::Up,
        Direction::Up => Direction::Down,
    })?;
    if !(0.0..=2.0).contains(&value) {
        return fallible!(NumericError, "erfc_inv operand is outside [0, 2]");
    }
    let value = match value {
        0.0 => f64::INFINITY,
        2.0 => f64::NEG_INFINITY,
        value => erfc_inv_f64(value),
    };
    approximate_endpoint(value, direction)
}

fn erfc(value: I) -> Fallible<I> {
    let (lo, hi) = value.into_endpoints();
    I::new(
        erfc_round(hi, Direction::Down)?,
        erfc_round(lo, Direction::Up)?,
    )
}

fn erfcx(value: I) -> Fallible<I> {
    let (lo, hi) = value.into_endpoints();
    I::new(
        erfcx_round(hi, Direction::Down)?,
        erfcx_round(lo, Direction::Up)?,
    )
}

fn erfc_inv(value: I) -> Fallible<I> {
    if value.lower_f64()? < 0.0 {
        return fallible!(NumericError, "erfc_inv interval reaches below zero");
    }
    if value.upper_f64()? > 2.0 {
        return fallible!(NumericError, "erfc_inv interval reaches above two");
    }
    let (lo, hi) = value.into_endpoints();
    I::new(
        erfc_inv_round(hi, Direction::Down)?,
        erfc_inv_round(lo, Direction::Up)?,
    )
}

#[inline]
fn nonnegative_sub(a: I, b: I) -> Fallible<I> {
    Ok((a - b)?.max(I::point(0.0)?))
}

/// Best-effort conservative upper bound on delta_mu(epsilon)
/// under idealized assumptions about Gaussian special-function error.
pub fn delta_via_gaussianDP(mu: f64, epsilon: f64) -> Fallible<f64> {
    check_mu(mu)?;
    check_epsilon(epsilon)?;
    if mu == 0.0 {
        return Ok(0.0);
    }

    let epsilon = I::point(epsilon)?;
    let mu = I::point(mu)?;
    let two = I::point(2.0)?;
    let half = I::point(0.5)?;
    let sqrt2 = I::between(SQRT_2.next_down(), SQRT_2.next_up())?;

    let u = (epsilon / mu.clone())?;
    let h = (mu / two)?;

    let x = ((u.clone() - h.clone())? / sqrt2.clone())?;
    let y = ((u + h)? / sqrt2)?;

    let delta = if x.is_nonnegative() {
        // delta = 0.5 * exp(-x^2) * (erfcx(x) - erfcx(y))
        let x2 = (x.clone() * x.clone())?;
        let common = (half.clone() * ((-x2).exp()?))?;
        let diff = nonnegative_sub(erfcx(x)?, erfcx(y)?)?;
        (common * diff)?
    } else {
        // delta = 0.5 * erfc(x) - 0.5 * exp(-x^2) * erfcx(y)
        let a = (half.clone() * erfc(x.clone())?)?;
        let x2 = (x.clone() * x)?;
        let b = ((half * ((-x2).exp()?))? * erfcx(y)?)?;
        nonnegative_sub(a, b)?
    };

    delta.clamp01()?.upper_f64()
}

/// Best-effort conservative lower bound on beta_mu(alpha)
/// under idealized assumptions about Gaussian special-function error.
pub fn beta_via_gaussianDP(mu: f64, alpha: f64) -> Fallible<f64> {
    check_mu(mu)?;
    check_alpha(alpha)?;

    if alpha == 0.0 {
        return Ok(1.0);
    }
    if alpha == 1.0 {
        return Ok(0.0);
    }

    let mu = I::point(mu)?;
    let alpha = I::point(alpha)?;
    let one = I::point(1.0)?;
    let two = I::point(2.0)?;
    let half = I::point(0.5)?;
    let sqrt2 = I::between(SQRT_2.next_down(), SQRT_2.next_up())?;

    // p = 2 * (1 - alpha), and beta is increasing in p.
    let p = (two * (one - alpha)?.clamp(0.0, 1.0)?)?.clamp(0.0, 2.0)?;

    if p.upper_f64()? <= 0.0 {
        return Ok(0.0);
    }
    if p.lower_f64()? >= 2.0 {
        return Ok(1.0);
    }

    // y = erfc_inv(p), and beta is decreasing in y.
    let y = erfc_inv(p)?;

    // q = mu / sqrt(2), and beta is decreasing in q.
    let q = (mu / sqrt2)?;

    // t = y + q, and beta = 0.5 * erfc(t) is decreasing in t.
    let beta = (half * erfc((y + q)?)?)?.clamp01()?;

    beta.lower_f64()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nonrepresentable_inputs_keep_antitone_endpoint_order() -> Fallible<()> {
        let input = (I::point(1.0)? + I::point(2.0_f64.powi(-54))?)?;
        assert_eq!(input.lower_f64()?, 1.0);
        assert_eq!(input.upper_f64()?, 1.0_f64.next_up());
        for (interval, lower_reference, upper_reference) in [
            (
                erfc(input.clone())?,
                erfc_f64(1.0_f64.next_up()),
                erfc_f64(1.0),
            ),
            (
                erfcx(input.clone())?,
                1.0_f64.next_up().erfcx(),
                1.0_f64.erfcx(),
            ),
            (
                erfc_inv(input)?,
                erfc_inv_f64(1.0_f64.next_up()),
                erfc_inv_f64(1.0),
            ),
        ] {
            assert!(interval.lower_f64()? <= lower_reference);
            assert!(interval.upper_f64()? >= upper_reference);
        }
        Ok(())
    }

    #[test]
    fn special_function_endpoints_and_invalid_values() -> Fallible<()> {
        assert_eq!(
            approximate_endpoint(1.0, Direction::Down)?.to_f64(Direction::Down)?,
            1.0_f64.next_down()
        );
        assert_eq!(
            approximate_endpoint(1.0, Direction::Up)?.to_f64(Direction::Up)?,
            1.0_f64.next_up()
        );
        for infinity in [f64::NEG_INFINITY, f64::INFINITY] {
            for direction in [Direction::Down, Direction::Up] {
                assert_eq!(
                    approximate_endpoint(infinity, direction)?.to_f64(direction)?,
                    infinity
                );
            }
        }
        assert!(approximate_endpoint(f64::NAN, Direction::Down).is_err());
        assert_eq!(erfc_inv(I::point(0.0)?)?.lower_f64()?, f64::INFINITY);
        assert_eq!(erfc_inv(I::point(2.0)?)?.upper_f64()?, f64::NEG_INFINITY);
        assert!(erfc_inv(I::point(-0.1)?).is_err());
        assert!(erfc_inv(I::point(2.1)?).is_err());
        assert_eq!(delta_via_gaussianDP(1.0, f64::INFINITY)?, 0.0);
        assert_eq!(delta_via_gaussianDP(0.0, 0.0)?, 0.0);
        assert_eq!(beta_via_gaussianDP(1.0, 0.0)?, 1.0);
        assert_eq!(beta_via_gaussianDP(1.0, 1.0)?, 0.0);
        assert!(beta_via_gaussianDP(1.0, 0.1)?.is_finite());
        Ok(())
    }
}
