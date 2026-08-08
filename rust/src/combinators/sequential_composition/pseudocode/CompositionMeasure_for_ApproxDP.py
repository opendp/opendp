# type: ignore
class CompositionMeasure(ApproxDP):
    def composability(  # |\label{line:composability}|
        self, adaptivity: Adaptivity
    ) -> Composability:
        return Composability.Concurrent

    def compose(self, d_mids: Vec[Self_Distance]) -> Self_Distance:
        eps_g, del_g = 0.0, 0.0
        for eps_i, del_i in d_mids:
            eps_g = eps_g.inf_add(eps_i)
            del_g = del_g.inf_add(del_i)
        return eps_g, del_g
