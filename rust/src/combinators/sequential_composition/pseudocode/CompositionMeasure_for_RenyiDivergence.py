# type: ignore
class CompositionMeasure(RenyiDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        # curves sharing an allocation are grouped as one (curve, k) pair,
        # in first-occurrence order, so that each distinct curve is
        # evaluated once, not once per copy
        groups = []  # Vec<(Self_Distance, u32)> |\label{line:groups}|
        for d_mid in d_mids:
            group = next(
                (group for group in groups
                 if Arc.ptr_eq(group[0].function, d_mid.function)),
                None,
            )
            if group is not None:
                group[1] += 1
            else:
                groups.push((d_mid, 1))

        def curve(alpha: float) -> float: # |\label{line:curve}|
            epsilons = [
                d_mid(alpha).inf_mul(k)  # |\label{line:inf-mul}|
                for d_mid, k in groups
            ]

            d_out = 0.0
            for d_mid in epsilons:
                d_out = d_out.inf_add(d_mid)
            return d_out

        return Function.new_fallible(curve)
        
