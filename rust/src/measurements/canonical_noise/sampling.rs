use dashu::{
    integer::{IBig, UBig},
    rational::RBig,
    rbig,
};

use super::CanonicalTradeoff;
use crate::{
    error::Fallible,
    traits::{Direction, samplers::InverseCDF},
};

pub(super) struct CanonicalRV<'a> {
    pub(super) shift: RBig,
    pub(super) scale: &'a RBig,
    pub(super) tradeoff: &'a CanonicalTradeoff,
}

impl InverseCDF for CanonicalRV<'_> {
    type Edge = RBig;

    fn inverse_cdf(
        &self,
        uniform: RBig,
        refinements: usize,
        direction: Direction,
    ) -> Fallible<Option<RBig>> {
        if uniform.is_zero() || uniform == RBig::ONE {
            return Ok(None);
        }
        let reflect = uniform > rbig!(1 / 2);
        let u = if reflect {
            RBig::ONE - uniform
        } else {
            uniform
        };
        let upper = (direction == Direction::Up) != reflect;
        let value = self.tradeoff.quantile(u, upper, refinements);
        let value = if reflect { -value } else { value };
        Ok(Some(value * self.scale + &self.shift))
    }
}

fn round_grid(value: RBig, grid: &RBig, upper: bool) -> RBig {
    let scaled = value * grid;
    RBig::from(if upper { scaled.ceil() } else { scaled.floor() }) / grid
}

impl CanonicalTradeoff {
    // Directed quantile on the lower half of the distribution.
    fn quantile(&self, mut u: RBig, upper: bool, refinements: usize) -> RBig {
        let c = &self.fixed_point;
        let width = RBig::ONE - rbig!(2) * c;
        if &u >= c {
            return (u - rbig!(1 / 2)) / width;
        }
        let mut offset = RBig::ZERO;
        // The tail drift is at least 2*u*width. Size the grid relative to
        // that scale and increase precision with each PSRN refinement.
        let bits = u.denominator().bit_len() + width.denominator().bit_len();
        let precision = 2 * bits + 64 + refinements;
        let grid = RBig::from_parts(IBig::from(UBig::ONE << precision), UBig::ONE);
        while &u < c {
            let index = self.segments.partition_point(|s| s.start <= u);
            let s = &self.segments[index - 1];
            let end = self.segments.get(index).map_or(c, |next| &next.start);
            if s.a.is_zero() {
                let steps = RBig::from(((end - &u) / &s.delta).ceil());
                u += &steps * &s.delta;
                offset -= steps;
                continue;
            }

            // T^n(u) = u + D(u)*S_n, where S_n is the geometric sum
            // and S_2n = 2*S_n + a*S_n^2.
            let double = |sum: &RBig| sum * (rbig!(2) + &s.a * sum);
            let limit = (end - &u) / s.eval(&u);
            let mut powers = Vec::new();
            let (mut lo, mut hi) = (RBig::ONE, RBig::ONE);
            let mut steps = RBig::ONE;
            while hi < limit {
                powers.push((steps.clone(), lo.clone(), hi.clone()));
                lo = round_grid(double(&lo), &grid, false);
                hi = round_grid(double(&hi), &grid, true);
                steps *= rbig!(2);
            }
            for (steps, lo, hi) in powers.into_iter().rev() {
                let d = s.eval(&u);
                if &u + &d * &hi >= *end {
                    continue;
                }
                let sum = if upper { hi } else { lo };
                let next = round_grid(&u + d * sum, &grid, upper).min((*end).clone());
                if next > u {
                    u = next;
                    offset -= steps;
                }
            }
            // An exact step ensures progress even when rounding prevents a jump.
            if &u < end {
                u += s.eval(&u);
                offset -= RBig::ONE;
            }
        }
        offset + (u - rbig!(1 / 2)) / width
    }
}
