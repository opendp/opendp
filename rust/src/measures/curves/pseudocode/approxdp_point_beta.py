# type: ignore
def approxdp_point_beta(
    epsilon: f64,
    delta: f64,
) -> Callable[[RBig], RBig]:
    epsilon = DInterval.point(epsilon)
    delta = RBig.try_from(delta)

    exp_eps = epsilon.exp().upper.to_rbig()  # `\label{exp-eps}`
    exp_neg_eps = (-epsilon).exp().lower.to_rbig()  # `\label{exp-neg-eps}`

    def tradeoff(alpha: RBig) -> RBig:  # `\label{tradeoff}`
        t1 = RBig(1) - delta - exp_eps * alpha
        base = max(RBig(1) - delta - alpha, RBig(0))
        t2 = exp_neg_eps * base
        return max(max(t1, t2), RBig(0))

    return tradeoff
