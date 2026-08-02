# type: ignore
def approximate_to_tradeoff(
    epsilon: f64,
    delta: f64,
) -> Callable[[RBig], RBig]:
    epsilon = FBig.try_from(epsilon)
    delta = RBig.try_from(delta)

    precision = epsilon.precision().max(10)
    epsilon = epsilon.with_precision(precision).value()

    exp_eps = RBig.try_from(epsilon.with_rounding().exp())  # `\label{exp-eps}`
    exp_neg_eps = RBig.try_from((-epsilon).with_rounding().exp())  # `\label{exp-neg-eps}`

    def tradeoff(alpha: RBig) -> RBig:  # `\label{tradeoff}`
        t1 = RBig(1) - delta - exp_eps * alpha
        base = max(RBig(1) - delta - alpha, RBig(0))
        t2 = exp_neg_eps * base
        return max(max(t1, t2), RBig(0))

    return tradeoff
