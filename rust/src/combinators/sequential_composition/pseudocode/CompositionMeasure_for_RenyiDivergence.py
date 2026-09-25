# type: ignore
class CompositionMeasure(RenyiDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[tuple[Self_Distance, u32]]) -> Self_Distance:
        # merge equal curves so that each is evaluated once, not once per copy
        groups = OrderedCounts()  # |\label{line:groups}|
        for d_mid, k_i in d_mids:
            groups.add(d_mid, k_i)

        def curve(alpha: float) -> float:  # |\label{line:curve}|
            epsilons = [
                d_mid(alpha).inf_mul(f64.from_(k))  # |\label{line:inf-mul}|
                for d_mid, k in groups
            ]

            d_out = 0.0
            for d_mid in epsilons:
                d_out = d_out.inf_add(d_mid)
            return d_out

        return Function.new_fallible(curve)
