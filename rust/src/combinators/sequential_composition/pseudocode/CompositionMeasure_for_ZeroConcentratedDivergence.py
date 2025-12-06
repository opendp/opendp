# type: ignore
class CompositionMeasure(ZeroConcentratedDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        match adaptivity:
            case Adaptivity.FullyAdaptive:
                return Composability.Sequential
            case _:
                return Composability.Concurrent

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        d_out = 0.0
        for d_mid in d_mids:
            d_out = d_out.inf_add(d_mid)
        return d_out
