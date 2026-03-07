# type: ignore
# analogous to impl NoiseThresholdPrivacyMap<L01InfDistance<AbsoluteDistance<RBig>>, Approximate<MaxDivergence>> for ZExpFamily<1> in Rust
class ZExpFamily1:
    def noise_threshold_privacy_map(
        self,
        _input_metric: L01InfDistance[AbsoluteDistance[RBig]],
        output_measure: Approximate[MaxDivergence],
        threshold: UBig,
    ) -> PrivacyMap[L01InfDistance[AbsoluteDistance[RBig]], Approximate[MaxDivergence]]:
        # |\label{line:noise-privacy-map}|
        noise_privacy_map = self.noise_privacy_map(L1Distance.default(), output_measure[0])
        scale = self.scale

        def privacy_map(d_in: tuple[u32, RBig, RBig]):
            l0, l1, li = d_in

            l1_sign, l1 = l1.floor().into_parts() # |\label{line:l1-floor}|
            if l1_sign != Sign.Positive: # |\label{line:l1-check}|
                raise f"l1 sensitivity ({l1}) must be non-negative"

            li_sign, li = li.floor().into_parts() # |\label{line:li-floor}|
            if li_sign != Sign.Positive: # |\label{line:li-check}|
                raise f"l-infinity sensitivity ({li}) must be non-negative"
            
            l1 = l1.min(li * l0)  # |\label{line:l1}|
            li = li.min(l1)  # |\label{line:li}|

            if l1.is_zero():  # |\label{line:zero-sens}|
                return 0.0, 0.0

            if scale.is_zero(): # |\label{line:zero-scale}|
                return f64.INFINITY, 1.0

            epsilon = noise_privacy_map.eval(l1) # |\label{line:epsilon}|
            
            if li > threshold: # |\label{line:threshold-check}|
                raise f"threshold must not be smaller than {li}"
            
            d_instability = threshold - li # |\label{line:distance-to-instability}|

            try:
                alpha_disc = conservative_discrete_laplacian_tail_to_alpha(
                    scale,
                    d_instability
                )
            except Exception:
                alpha_disc = None
            
            try:
                alpha_cont = conservative_continuous_laplacian_tail_to_alpha(
                    scale,
                    d_instability,
                )
            except Exception:
                alpha_cont = None

            delta_single = option_min(alpha_disc, alpha_cont)
            if delta_single is None:
                raise "failed to compute tail bound in privacy map"

            delta_joint: f64 = (1.0).inf_sub(
                (1.0).neg_inf_sub(delta_single).neg_inf_powi(IBig.from_(l0)),
            )

            # delta is only sensibly at most 1
            return epsilon, delta_joint.min(1.0)

        return PrivacyMap.new_fallible(privacy_map)
