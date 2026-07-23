//! Interval arithmetic for testing that privacy-sensitive computations round conservatively.

use dashu::{
    base::SquareRoot,
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
};

use crate::traits::ToFloatRounded;

// interval widths (~2^-150 relative) are negligible against f64 rounding (~2^-52 relative)
const PRECISION: usize = 150;

/// A certified bracket lo <= true value <= hi, maintained under directed rounding.
#[derive(Clone)]
pub struct Interval {
    lo: FBig<Down>,
    hi: FBig<Up>,
}

impl Interval {
    pub fn from_f64(v: f64) -> Self {
        Interval {
            lo: FBig::<Down>::try_from(v)
                .unwrap()
                .with_precision(PRECISION)
                .value(),
            hi: FBig::<Up>::try_from(v)
                .unwrap()
                .with_precision(PRECISION)
                .value(),
        }
    }

    pub fn from_rational(v: &RBig) -> Self {
        Self::from_rational_bounds(v, v)
    }

    /// For values only known to lie within [lo, hi].
    pub fn from_rational_bounds(lo: &RBig, hi: &RBig) -> Self {
        assert!(lo <= hi);
        let (lo_num, lo_den) = lo.clone().into_parts();
        let (hi_num, hi_den) = hi.clone().into_parts();
        Interval {
            lo: FBig::<Down>::from(lo_num).with_precision(PRECISION).value()
                / FBig::<Down>::from(lo_den),
            hi: FBig::<Up>::from(hi_num).with_precision(PRECISION).value()
                / FBig::<Up>::from(hi_den),
        }
    }

    // with_rounding changes only the type parameter, never the value
    fn hi_down(&self) -> FBig<Down> {
        self.hi.clone().with_rounding()
    }
    fn lo_up(&self) -> FBig<Up> {
        self.lo.clone().with_rounding()
    }

    pub fn neg(&self) -> Self {
        Interval {
            lo: (-self.hi.clone()).with_rounding(),
            hi: (-self.lo.clone()).with_rounding(),
        }
    }

    pub fn add(&self, o: &Self) -> Self {
        Interval {
            lo: self.lo.clone() + o.lo.clone(),
            hi: self.hi.clone() + o.hi.clone(),
        }
    }

    pub fn sub(&self, o: &Self) -> Self {
        self.add(&o.neg())
    }

    pub fn mul(&self, o: &Self) -> Self {
        let lo = [
            self.lo.clone() * o.lo.clone(),
            self.lo.clone() * o.hi_down(),
            self.hi_down() * o.lo.clone(),
            self.hi_down() * o.hi_down(),
        ]
        .into_iter()
        .min()
        .unwrap();
        let hi = [
            self.lo_up() * o.lo_up(),
            self.lo_up() * o.hi.clone(),
            self.hi.clone() * o.lo_up(),
            self.hi.clone() * o.hi.clone(),
        ]
        .into_iter()
        .max()
        .unwrap();
        Interval { lo, hi }
    }

    pub fn div(&self, o: &Self) -> Self {
        assert!(
            o.lo > FBig::<Down>::ZERO || o.hi < FBig::<Up>::ZERO,
            "denominator bracket must not contain zero"
        );
        let lo = [
            self.lo.clone() / o.lo.clone(),
            self.lo.clone() / o.hi_down(),
            self.hi_down() / o.lo.clone(),
            self.hi_down() / o.hi_down(),
        ]
        .into_iter()
        .min()
        .unwrap();
        let hi = [
            self.lo_up() / o.lo_up(),
            self.lo_up() / o.hi.clone(),
            self.hi.clone() / o.lo_up(),
            self.hi.clone() / o.hi.clone(),
        ]
        .into_iter()
        .max()
        .unwrap();
        Interval { lo, hi }
    }

    pub fn exp(&self) -> Self {
        Interval {
            lo: self.lo.clone().exp(),
            hi: self.hi.clone().exp(),
        }
    }

    pub fn ln(&self) -> Self {
        assert!(
            self.lo > FBig::<Down>::ZERO,
            "ln requires a positive bracket"
        );
        Interval {
            lo: self.lo.clone().ln(),
            hi: self.hi.clone().ln(),
        }
    }

    pub fn sqrt(&self) -> Self {
        assert!(
            self.lo >= FBig::<Down>::ZERO,
            "sqrt requires a non-negative bracket"
        );
        Interval {
            lo: self.lo.clone().sqrt(),
            hi: self.hi.clone().sqrt(),
        }
    }

    pub fn strictly_below(&self, v: f64) -> bool {
        self.hi < FBig::<Up>::try_from(v).unwrap()
    }

    pub fn strictly_above(&self, v: f64) -> bool {
        self.lo > FBig::<Down>::try_from(v).unwrap()
    }
}

/// `value` must strictly exceed the bracketed truth, and `mirrored`
/// (the same chain with every rounding flipped) must fall strictly below it.
pub fn assert_rounds_up(value: f64, mirrored: f64, truth: &Interval) {
    assert!(
        truth.strictly_below(value),
        "value ({value:e}) must strictly exceed the true value (<= {:e})",
        truth.hi.clone().to_f64_rounded(),
    );
    assert!(
        truth.strictly_above(mirrored),
        "mirrored value ({mirrored:e}) must fall strictly below the true value (>= {:e})",
        truth.lo.clone().to_f64_rounded(),
    );
    assert!(mirrored < value);
}

#[cfg(test)]
mod test {
    use super::*;
    use dashu::rbig;

    #[test]
    fn test_interval_brackets_truth() {
        // exp(-1/10)
        let one = Interval::from_f64(1.0);
        let ten = Interval::from_f64(10.0);
        let i = one.div(&ten).neg().exp();
        let approx = (-0.1f64).exp();
        assert!(i.lo < FBig::<Down>::try_from(approx.next_up()).unwrap());
        assert!(i.hi > FBig::<Up>::try_from(approx.next_down()).unwrap());

        // [-3, 2] * [-5, 7] = [-21, 15]
        let a = Interval::from_rational_bounds(&rbig!(-3), &rbig!(2));
        let b = Interval::from_rational_bounds(&rbig!(-5), &rbig!(7));
        let p = a.mul(&b);
        assert!(p.lo <= FBig::<Down>::try_from(-21.0).unwrap());
        assert!(p.hi >= FBig::<Up>::try_from(15.0).unwrap());

        let s = Interval::from_f64(2.0).sqrt();
        assert!(s.strictly_above(1.4) && s.strictly_below(1.5));

        // [1, 2] - [0, 3] = [-2, 2]
        let d = Interval::from_rational_bounds(&rbig!(1), &rbig!(2))
            .sub(&Interval::from_rational_bounds(&rbig!(0), &rbig!(3)));
        assert!(d.lo <= FBig::<Down>::try_from(-2.0).unwrap());
        assert!(d.hi >= FBig::<Up>::try_from(2.0).unwrap());
    }
}
