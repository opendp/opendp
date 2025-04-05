# type: ignore
def find_nearest_multiple_of_2k(x: RBig, k: i32) -> IBig:
    # exactly compute x/2^k and break into fractional parts
    num, den = x_mul_2k(x, -k).into_parts() # `\label{line:into-parts}`

    # argmin_i |i * 2^k - x|, the index of nearest multiple of 2^k
    return (floor_div(num << 1, den) + 1) >> 1 # `\label{line:return}`
