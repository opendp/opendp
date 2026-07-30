# type: ignore
class ZExpFamily2:
    def noise_privacy_map(
        self, _input_metric: L2Distance[RBig], _output_measure: PrivacyCurveDP
    ) -> PrivacyMap[L2Distance[RBig], PrivacyCurveDP]:
        scale = self.scale
        if scale < RBig.ZERO:  # |\label{line:neg-scale}|
            raise "scale must be non-negative"

        def privacy_map(d_in: RBig):
            if d_in < RBig.ZERO:  # |\label{line:neg-sens}|
                raise "sensitivity must be non-negative"
            if d_in == RBig.ZERO:  # |\label{line:zero-sens}|
                return PrivacyCurve().with_zCDP(0.0)
            if scale == RBig.ZERO:  # |\label{line:zero-scale}|
                return PrivacyCurve().with_zCDP(float("inf"))

            rho = f64.inf_cast((d_in / scale) ** 2 / RBig(2))  # |\label{line:rho}|
            return PrivacyCurve().with_zCDP(rho)  # |\label{line:curve}|

        return PrivacyMap.new_fallible(privacy_map)
