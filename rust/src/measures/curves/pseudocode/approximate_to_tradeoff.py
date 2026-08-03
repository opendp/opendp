# type: ignore
def beta(
    self: ApproxDPPoint,
    alpha: RBig,
) -> RBig:
    t1 = self.one_minus_delta - self.exp_eps_up * alpha
    base = max(self.one_minus_delta - alpha, RBig(0))
    t2 = self.exp_neg_eps_down * base
    return max(max(t1, t2), RBig(0))
