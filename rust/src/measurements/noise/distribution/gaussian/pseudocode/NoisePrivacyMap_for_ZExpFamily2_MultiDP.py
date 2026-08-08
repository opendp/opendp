# type: ignore
class ZExpFamily2:
    def noise_privacy_map(
        self, _input_metric: L2Distance[RBig], _output_measure: MultiDP
    ) -> PrivacyMap[L2Distance[RBig], MultiDP]:
        zcdp_map = self.noise_privacy_map(  # |\label{line:zcdp-map}|
            L2Distance.default(), zCDP
        )

        def privacy_map(d_in: RBig):
            rho = zcdp_map(d_in)
            return PrivacyGuarantee().with_zCDP(rho, 0.0)  # |\label{line:curve}|

        return PrivacyMap.new_fallible(privacy_map)
