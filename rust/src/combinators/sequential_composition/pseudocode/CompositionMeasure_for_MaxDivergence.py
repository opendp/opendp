# type: ignore
class CompositionMeasure(MaxDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[tuple[Self_Distance, u32]]) -> Self_Distance:
        d_out = 0.0
        for d_mid, k_i in d_mids:
            d_out = d_out.inf_add(d_mid.inf_mul(f64.from_(k_i)))
        return d_out
