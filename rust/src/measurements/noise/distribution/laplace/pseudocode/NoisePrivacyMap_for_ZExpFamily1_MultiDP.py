# type: ignore
class ZExpFamily1:
    def noise_privacy_map(self, _metric, _measure):
        scale = self.scale
        if scale < RBig.ZERO:  # |\label{line:neg-scale}|
            raise "scale must be non-negative"
        if scale == RBig.ZERO:  # |\label{line:zero-scale}|
            raise "zero scale is unsupported for MultiDP"

        def privacy_map(d_in):
            if d_in < RBig.ZERO:  # |\label{line:neg-sens}|
                raise "sensitivity must be non-negative"
            if d_in == RBig.ZERO:  # |\label{line:zero-sens}|
                return (
                    PrivacyGuarantee()
                    .with_approx_dp(0.0, 0.0)
                    .with_zCDP(0.0, 0.0)
                    .with_renyiDP(lambda orders: [0.0 for _ in orders], 0.0)
                )
            epsilon_exact = d_in / scale
            epsilon = f64.inf_cast(epsilon_exact)
            if not epsilon.is_finite():
                raise "privacy parameters are not finite"

            rho = zcdp_discrete_laplace(epsilon_exact, d_in, scale).or_else(lambda: epsilon)
            curve = PrivacyGuarantee().with_approx_dp(epsilon, 0.0).with_zCDP(rho, 0.0)

            def rdp(orders):
                values = []
                for alpha in orders:
                    check_renyi_order(alpha)
                    values.append(
                        rdp_discrete_laplace(alpha, d_in, scale)
                        .or_else(lambda: rdp_from_pureDP(alpha, epsilon))
                        .or_else(lambda: epsilon)
                    )
                return values
            return curve.with_renyiDP(rdp, 0.0)

        return PrivacyMap.new_fallible(privacy_map)
