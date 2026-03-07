# type: ignore
class CompositionMeasure(MaxDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        if matches(adaptivity, Adaptivity.FullyAdaptive):
            raise "fully-adaptive composition is not currently supported for max-divergence"
        return Composability.Concurrent

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        d_out = 0.0
        for d_mid in d_mids:
            d_out = d_out.inf_add(d_mid)
        return d_out
