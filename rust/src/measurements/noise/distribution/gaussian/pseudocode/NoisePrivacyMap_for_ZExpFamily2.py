# type: ignore
# analogous to impl NoisePrivacyMap<L2Distance<RBig>, ZeroConcentratedDivergence> for ZExpFamily<1> in Rust
class ZExpFamily2:
    def noise_privacy_map(
        self, _input_metric: L2Distance[RBig], _output_measure: ZeroConcentratedDivergence
    ) -> PrivacyMap[L2Distance[RBig], ZeroConcentratedDivergence]:
        scale = self.scale
        if scale < RBig.ZERO:  # |\label{line:neg-scale}|
            raise "scale must be non-negative"

        def privacy_map(d_in: RBig):
            if d_in < RBig.ZERO:  # |\label{line:neg-sens}|
                raise "sensitivity must be non-negative"

            if d_in.is_zero():  # |\label{line:zero-sens}|
                return 0.0

            if scale.is_zero():  # |\label{line:zero-scale}|
                return float("inf")

            return f64.inf_cast((d_in / scale).pow(2) / rbig(2))  # |\label{line:map}|

        return PrivacyMap.new_fallible(privacy_map)
