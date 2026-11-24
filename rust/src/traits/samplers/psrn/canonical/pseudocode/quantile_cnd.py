# type: ignore
def quantile_cnd(uniform: RBig, f: Callable[[RBig], RBig], c: RBig) -> RBig | None:
    if uniform.is_zero() or uniform == RBig(1):
        return None

    den = RBig(1) - RBig(2) * c
    if den.is_zero():
        return None

    offset = RBig(0)
    while True:
        if uniform.is_zero() or uniform == RBig(1):
            return None
        if uniform < c:
            next_uniform = RBig(1) - f(uniform)
            if next_uniform == uniform:
                return None
            uniform = next_uniform
            offset -= RBig(1)
        elif uniform <= RBig(1) - c:
            return offset + (uniform - RBig(1, 2)) / den
        else:
            next_uniform = f(RBig(1) - uniform)
            if next_uniform == uniform:
                return None
            uniform = next_uniform
            offset += RBig(1)
