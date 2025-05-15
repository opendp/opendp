# type: ignore
class CompositionMeasure(ApproximateZeroConcentratedDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Sequential

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        rho_g, del_g = 0.0, 0.0
        for rho_i, del_i in d_mids:
            rho_g = rho_g.inf_add(rho_i)
            del_g = del_g.inf_add(del_i)
        return rho_g, del_g
