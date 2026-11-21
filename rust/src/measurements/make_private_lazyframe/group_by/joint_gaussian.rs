//! An opt-in isotropic discrete-Gaussian strategy for exact integer group-by
//! statistics. The strategy is not selected by the public API until the
//! workload strategy-selection phase.
//
//! Every coordinate is an exact integer count or a bounded Int32 sum. The
//! existing scalar compiler checks overflow and supplies the same independent
//! discrete Gaussian noise and missing-group fills as before. The only new
//! privacy accounting is the joint L2 bound over all groups and coordinates.
//!
//! For an ideal query vector q and independent discrete Gaussian noise with
//! the same scale sigma on every integer coordinate, the joint release is
//! rho-zCDP with rho <= ||q(D)-q(D')||_2^2 / (2*sigma^2).
//! The bound here applies because each admitted aggregate computes an *exact*
//! integer statistic. Float sums and centered Float64 second moments are
//! deliberately excluded until their rounding-relaxation analysis is complete.

use dashu::{rational::RBig, rbig};
use polars::prelude::{DataType, Expr};

use crate::domains::{Invariant, Margin, SeriesDomain};
use crate::error::Fallible;
use crate::measurements::{
    expr_dp_counting_query::DPLenShim,
    expr_dp_frame_len::DPFrameLenShim,
    expr_dp_sum::DPSumShim,
};
use crate::polars::{apply_plugin, literal_value_of, match_shim};
use crate::traits::{InfCast, InfMul};

use super::joint_sensitivity::IdealJointSensitivity;

#[derive(Clone)]
pub(super) struct JointGaussian {
    pub aggs: Vec<Expr>,
    pub sigma: f64,
    sensitivity: IdealJointSensitivity,
}

impl JointGaussian {
    /// Return None unless the *entire* workload uses compatible, implicitly
    /// calibrated integer mechanisms. Rejecting mixed workloads is essential:
    /// omitted coordinates must not escape the joint privacy map.
    pub fn prepare(
        aggs: &[Expr],
        columns: &[SeriesDomain],
        margin: &Margin,
        global_scale: Option<f64>,
    ) -> Fallible<Option<Self>> {
        let Some(global_scale) = global_scale else {
            return Ok(None);
        };
        if global_scale.is_sign_negative() || !global_scale.is_finite() {
            return crate::fallible!(MakeMeasurement, "joint Gaussian scale must be finite and nonnegative");
        }
        if aggs.len() < 2 {
            return Ok(None);
        }
        let Some(sensitivity) = IdealJointSensitivity::from_aggs(aggs, columns, &[])? else {
            return Ok(None);
        };

        // Calibrate at the smallest admissible dataset distance, matching
        // approximate_c_stability's convention for bounded/unbounded DP.
        let min_distance = if matches!(margin.invariant, Some(Invariant::Lengths)) {
            2
        } else {
            1
        };
        let (l0, li) = (margin.l_0(min_distance), margin.l_inf(min_distance));
        let (_, base_scale) = sensitivity.ideal_bounds((l0, min_distance, li))?;
        if !base_scale.is_finite() || base_scale == 0.0 {
            return Ok(None);
        }
        let sigma = base_scale.inf_mul(&global_scale)?;
        if !sigma.is_finite() {
            return crate::fallible!(MakeMeasurement, "joint Gaussian noise scale overflows");
        }

        let mut rewritten = Vec::with_capacity(aggs.len());
        for original in aggs {
            let name = original.clone().meta().output_name()?;
            let expr = original.clone().meta().undo_aliases();

            // A frame count has one integer contribution per record.
            if let Some([scale, signed]) = match_shim::<DPFrameLenShim, 2>(&expr)? {
                if literal_value_of::<f64>(&scale)?.is_some()
                    || !matches!(&signed, Expr::Literal(value) if value.bool() == Some(false))
                {
                    return Ok(None);
                }
                rewritten.push(
                    apply_plugin(vec![polars::prelude::lit(base_scale), signed],
                        expr, DPFrameLenShim).alias(name.as_str()),
                );
                continue;
            }

            if let Some([input, scale, signed]) = match_shim::<DPLenShim, 3>(&expr)? {
                if !matches!(&input, Expr::Column(_))
                    || literal_value_of::<f64>(&scale)?.is_some()
                    || !matches!(signed, Expr::Literal(ref value) if value.bool() == Some(false))
                {
                    return Ok(None);
                }
                rewritten.push(
                    apply_plugin(vec![input, polars::prelude::lit(base_scale), signed],
                        expr, DPLenShim).alias(name.as_str()),
                );
                continue;
            }

            if let Some([input, lower, upper, scale]) = match_shim::<DPSumShim, 4>(&expr)? {
                let Expr::Column(column_name) = &input else {
                    return Ok(None);
                };
                if !columns.iter().any(|c| {
                    c.name.as_str() == column_name.as_str() && c.dtype() == DataType::Int32
                }) || literal_value_of::<i32>(&lower)?.is_none()
                    || literal_value_of::<i32>(&upper)?.is_none()
                    || literal_value_of::<f64>(&scale)?.is_some()
                {
                    return Ok(None);
                }
                rewritten.push(
                    apply_plugin(vec![input, lower, upper, polars::prelude::lit(base_scale)],
                        expr, DPSumShim).alias(name.as_str()),
                );
                continue;
            }

            return Ok(None);
        }

        Ok(Some(Self { aggs: rewritten, sigma, sensitivity }))
    }

    /// Privacy map for the *entire* product of independent Gaussian noises
    /// across groups and coordinates. The rational calculation matches the
    /// established Gaussian noise privacy map, with upward f64 conversion.
    pub fn rho(&self, distance: (u32, u32, u32)) -> Fallible<f64> {
        let (_, delta) = self.sensitivity.ideal_bounds(distance)?;
        if delta == 0.0 {
            return Ok(0.0);
        }
        if self.sigma == 0.0 {
            return Ok(f64::INFINITY);
        }
        let delta = RBig::try_from(delta)?;
        let sigma = RBig::try_from(self.sigma)?;
        f64::inf_cast((delta / sigma).pow(2) / rbig!(2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::AtomDomain;
    use crate::polars::{dp_len, PrivacyNamespace};
    use polars::prelude::{col, lit};

    #[test]
    fn test_joint_integer_noise_and_privacy() -> Fallible<()> {
        let columns = vec![SeriesDomain::new("x", AtomDomain::<i32>::default())];
        let margin = Margin::by(["key"]).with_max_length(3).with_max_groups(2);
        let aggs = vec![
            dp_len(None, false).alias("n"),
            col("x").dp().sum((lit(0i32), lit(4i32)), None).alias("s"),
        ];
        let joint = JointGaussian::prepare(&aggs, &columns, &margin, Some(2.0))?
            .expect("exact integer workload");
        assert_eq!(joint.aggs.len(), 2);
        assert!(joint.sigma >= (17.0f64).sqrt() * 2.0);
        let rho = joint.rho((1, 1, 1))?;
        assert!(rho >= 0.125 && rho < 0.126);
        let larger = joint.rho((2, 2, 2))?;
        assert!(larger >= 4.0 * rho - 1e-12);
        Ok(())
    }

    #[test]
    fn test_disallows_float_or_explicit_scale() -> Fallible<()> {
        let margin = Margin::by(["key"]).with_max_length(3).with_max_groups(2);
        let floats = vec![SeriesDomain::new("x", AtomDomain::<f64>::default())];
        let aggs = vec![
            dp_len(None, false).alias("n"),
            col("x").dp().sum((lit(0.0), lit(4.0)), None).alias("s"),
        ];
        assert!(JointGaussian::prepare(&aggs, &floats, &margin, Some(1.0))?.is_none());

        let ints = vec![SeriesDomain::new("x", AtomDomain::<i32>::default())];
        let aggs = vec![
            dp_len(None, false).alias("n"),
            col("x").dp().sum((lit(0i32), lit(4i32)), Some(1.0)).alias("s"),
        ];
        assert!(JointGaussian::prepare(&aggs, &ints, &margin, Some(1.0))?.is_none());
        Ok(())
    }

    #[test]
    fn test_bounded_adjacency_uses_two_records() -> Fallible<()> {
        let columns = vec![SeriesDomain::new("x", AtomDomain::<i32>::default())];
        let margin = Margin::by(["key"])
            .with_invariant_lengths()
            .with_max_length(3)
            .with_max_groups(2);
        let aggs = vec![
            dp_len(None, false).alias("n"),
            col("x").dp().sum((lit(0i32), lit(4i32)), None).alias("s"),
        ];
        let joint = JointGaussian::prepare(&aggs, &columns, &margin, Some(1.0))?
            .expect("integer workload");
        assert!(joint.sigma >= (68.0f64).sqrt());
        Ok(())
    }
}
