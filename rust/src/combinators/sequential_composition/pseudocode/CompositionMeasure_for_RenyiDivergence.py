# type: ignore
class CompositionMeasure(RenyiDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        def curve(alpha: float) -> float:  # |\label{line:curve}|
            epsilons = [d_mid(alpha) for d_mid in d_mids]

            d_out = 0.0
            for d_mid in epsilons:
                d_out = d_out.inf_add(d_mid)
            return d_out

        return Function.new_fallible(curve)

    def new_acc(self) -> RenyiAcc:
        return RenyiAcc(groups=[], indices={})  # |\label{line:new-acc}|

    def accumulate_keyed(
        self, acc: RenyiAcc, key: MapKey, eval: Callable[[], Self_Distance]
    ):
        # charges of the same privacy map share a group,
        # so the map is evaluated once, not once per charge
        if key in acc.indices:
            i = acc.indices[key]
        else:
            # if eval() raises, no group is started
            acc.groups.push((eval(), 0))  # |\label{line:new-group}|
            i = acc.indices[key] = len(acc.groups) - 1
        # raises on overflow, before the count is changed
        acc.groups[i].k = acc.groups[i].k.alerting_add(1)  # |\label{line:count}|

    def compose_acc(self, acc: RenyiAcc) -> Self_Distance:  # |\label{line:compose-acc}|
        groups = acc.groups.clone()

        def curve(alpha: f64) -> f64:
            d_out = 0.0
            for d_mid, k in groups:
                eps = d_mid(alpha).inf_mul(f64(k))  # |\label{line:inf-mul}|
                d_out = d_out.inf_add(eps)
            return d_out

        return Function.new_fallible(curve)
