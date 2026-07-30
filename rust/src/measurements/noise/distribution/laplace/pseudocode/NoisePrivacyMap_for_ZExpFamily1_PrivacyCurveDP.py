# type: ignore
class ZExpFamily1:
    def noise_privacy_map(self, _metric, _measure):
        scale = self.scale
        if scale < RBig.ZERO:  # |\label{line:neg-scale}|
            raise "scale must be non-negative"

        def privacy_map(d_in):
            if d_in < RBig.ZERO:  # |\label{line:neg-sens}|
                raise "sensitivity must be non-negative"
            if d_in == RBig.ZERO:  # |\label{line:zero-sens}|
                return PrivacyCurve().with_approxDP([(0.0, 0.0)]).with_zCDP(0.0)
            if scale == RBig.ZERO:  # |\label{line:zero-scale}|
                raise "no finite privacy guarantee"

            epsilon = f64.inf_cast(d_in / scale)
            sensitivity = f64.inf_cast(d_in)
            scale_f = f64.inf_cast(scale)
            if not epsilon.is_finite() or not sensitivity.is_finite() or scale_f <= 0:
                raise "privacy parameters are not finite"

            rho = zcdp_discrete_laplace(epsilon, sensitivity, scale_f)
            curve = PrivacyCurve().with_approxDP([(epsilon, 0.0)]).with_zCDP(rho)
            return curve.with_renyiDP(
                lambda alpha: rdp_discrete_laplace(alpha, sensitivity, scale_f)
            )

        return PrivacyMap.new_fallible(privacy_map)
