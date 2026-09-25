# type: ignore
class CompositionMeasure(ApproximateMaxDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[tuple[Self_Distance, u32]]) -> Self_Distance:
        eps_g, del_g = 0.0, 0.0
        for (eps_i, del_i), k_i in d_mids:
            k_i = f64.from_(k_i)
            eps_g = eps_g.inf_add(eps_i.inf_mul(k_i))
            del_g = del_g.inf_add(del_i.inf_mul(k_i))
        return eps_g, del_g
