# type: ignore
def exponential_top_k(x: list[TIA], scale: RBig, k: usize, negate: bool):
    sign = Sign.from_(negate)
    scale = scale.into_rational()

    y = [x_i.into_rational() * sign for x_i in x]  # `\label{negate}`
    return peel_permute_and_flip(y, scale, k)
