# type: ignore
def noisy_top_k(
    x: list[TIA],
    scale: f64,
    k: usize,
    negate: bool,
    replacement: bool,
):
    sign = Sign.from_(negate)
    scale = scale.into_rational()

    y = [  # `\label{negate}`
        x_i.total_clamp(TIA.MIN_FINITE, TIA.MAX_FINITE).into_rational() * sign
        for x_i in x
    ]
    return peel_permute_and_flip(y, scale, k, replacement)
