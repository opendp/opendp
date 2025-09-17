# type: ignore
def x_mul_2k(x: RBig, k: i32) -> RBig:
    num, den = x.into_parts()  # |\label{line:into-parts}|
    if k < 0:
        den <<= cast(-k, usize)
    else:
        num <<= cast(k, usize)

    return RBig.from_parts(num, den)
