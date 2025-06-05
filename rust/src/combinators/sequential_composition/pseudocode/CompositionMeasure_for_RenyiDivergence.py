# type: ignore
class CompositionMeasure(RenyiDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        def curve(alpha: float) -> float: # |\label{line:curve}|
            epsilons = [d_mid(alpha) for d_mid in d_mids]

            d_out = 0.0
            for d_mid in epsilons:
                d_out = d_out.inf_add(d_mid)
            return d_out

        return Function.new_fallible(curve)
        
