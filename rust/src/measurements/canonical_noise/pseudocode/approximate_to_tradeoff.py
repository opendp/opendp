# type: ignore
def appproximate_to_tradeoff(
    param: tuple[f64, f64]
) -> tuple[Callable[[RBig], RBig], RBig]:
    epsilon, delta = param

    exp_eps = epsilon.with_rounding(Down).exp()  # `\label{exp-eps}`
    exp_eps = RBig.try_from(exp_eps)

    exp_neg_eps = (-epsilon).with_rounding(Up).exp()  # `\label{exp-neg-eps}`
    exp_neg_eps = RBig.try_from(exp_neg_eps)

    fixed_point = (RBig(1) - delta) / (RBig(1) + exp_eps)

    if fixed_point >= RBig(1, 2):
        raise ValueError("fixed point of tradeoff curve must be less than 1/2")

    def tradeoff(alpha: RBig) -> RBig:  # `\label{tradeoff}`
        t1 = RBig(1) - delta - exp_eps * alpha
        t2 = exp_neg_eps * (RBig(1) - delta - alpha)
        return max(max(t1, t2), RBig(0))

    return tradeoff, fixed_point
