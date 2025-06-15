# type: ignore
def quantile_cnd(
    uniform: RBig, f: Callable[[RBig], RBig], c: RBig
) -> RBig | None:
    if uniform < c:
        return quantile_cnd(RBig(1) - f(uniform), f, c) - RBig(1)
    elif uniform <= RBig(1) - c:  # the linear function
        num = uniform - RBig(1, 2)
        den = RBig(1) - RBig(2) * c
        if den.is_zero():
            return
        return num / den
    else:
        return quantile_cnd(f(RBig(1) - uniform), f, c) + RBig(1)
