# type: ignore
class CompositionMeasure(ApproximateZeroConcentratedDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Sequential

    def compose(self, d_mids: Vec[tuple[Self_Distance, u32]]) -> Self_Distance:
        rho_g, del_g = 0.0, 0.0
        for (rho_i, del_i), k_i in d_mids:
            k_i = f64.from_(k_i)
            rho_g = rho_g.inf_add(rho_i.inf_mul(k_i))
            del_g = del_g.inf_add(del_i.inf_mul(k_i))
        return rho_g, del_g
