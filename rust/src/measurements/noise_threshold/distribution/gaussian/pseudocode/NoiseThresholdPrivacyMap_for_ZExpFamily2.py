# type: ignore
# analogous to impl NoiseThresholdPrivacyMap<L02I<AbsoluteDistance<RBig>>, Approximate<ZeroConcentratedDivergence>> for ZExpFamily<2> in Rust
class ZExpFamily2:
    def noise_threshold_privacy_map(
        self,
        _input_metric: L02InfDistance[AbsoluteDistance[RBig]],
        output_measure: Approximate[ZeroConcentratedDivergence],
        threshold: UBig,
    ) -> PrivacyMap[L02InfDistance[AbsoluteDistance[RBig]], Approximate[ZeroConcentratedDivergence]]:
        # |\label{line:noise-privacy-map}|
        noise_privacy_map = self.noise_privacy_map(L2Distance.default(), output_measure[0])
        scale = self.scale

        def privacy_map(d_in: tuple[u32, RBig, RBig]):
            l0, l2, li = d_in

            li_sign, li = li.floor().into_parts() # |\label{line:li-floor}|
            if li_sign != Sign.Positive: # |\label{line:li-check}|
                raise f"l-infinity sensitivity ({li}) must be non-negative"
            
            l2 = l2.min(li * RBig.try_from(f64.from_(l0).inf_sqrt()))  # |\label{line:l2}|
            li = li.min(l2.floor())  # |\label{line:li}|

            if l2.is_zero():  # |\label{line:zero-sens}|
                return 0.0, 0.0

            if scale.is_zero(): # |\label{line:zero-scale}|
                return f64.INFINITY, 1.0

            rho = noise_privacy_map.eval(l2) # |\label{line:rho}|
            
            if li > threshold: # |\label{line:threshold-check}|
                raise f"threshold must not be smaller than {li}"
            
            d_instability = threshold - li # |\label{line:distance-to-instability}|

            delta_single = conservative_discrete_gaussian_tail_to_alpha(
                scale,
                d_instability,
            )

            delta_joint: f64 = (1.0).inf_sub(
                (1.0).neg_inf_sub(delta_single).neg_inf_powi(IBig.from_(l0)),
            )

            # delta is only sensibly at most 1
            return rho, delta_joint.min(1.0)

        return PrivacyMap.new_fallible(privacy_map)
