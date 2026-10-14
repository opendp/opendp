# type: ignore
class CompositionMeasure(RenyiDivergence):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[tuple[Self_Distance, u32]]) -> Self_Distance:
        def curve(alpha: f64) -> f64:  # |\label{line:curve}|
            epsilons = [
                d_mid(alpha).inf_mul(f64(k))  # |\label{line:inf-mul}|
                for d_mid, k in d_mids
            ]

            d_out = 0.0
            for eps in epsilons:
                d_out = d_out.inf_add(eps)
            return d_out

        return Function.new_fallible(curve)

    def new_acc(self) -> RenyiAcc:
        return RenyiAcc(OrderedCounts())  # |\label{line:new-acc}|

    def accumulate_keyed(
        self, acc: RenyiAcc, key: MapKey, eval: Callable[[], Self_Distance]
    ):
        # charges of the same privacy map share a group,
        # so the map is evaluated once, not once per charge
        acc.counts.add(key, 1, eval)  # |\label{line:accumulate}|

    def compose_acc(self, acc: RenyiAcc) -> Self_Distance:
        return self.compose(list(acc.counts.iter()))  # |\label{line:compose-acc}|


class OrderedCounts:
    """(value, count) groups, indexed by key, in the order keys are first added"""

    def __init__(self):
        self.groups = []  # Vec<(V, u32)>
        self.indices = {}  # HashMap<K, usize>

    def add(self, key: K, k: u32, value: Callable[[], V]):  # |\label{line:add}|
        if key in self.indices:
            i = self.indices[key]
        else:
            # if value() raises, no group is started
            self.groups.push((value(), 0))  # |\label{line:new-group}|
            i = self.indices[key] = len(self.groups) - 1
        # raises on overflow, before the count is changed
        count = self.groups[i].count.alerting_add(k)  # |\label{line:count}|
        self.groups[i].count = count

    def iter(self):
        return iter(self.groups)
