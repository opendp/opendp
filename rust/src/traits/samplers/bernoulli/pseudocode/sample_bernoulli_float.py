# type: ignore
# returns a single bit with some probability of success
def sample_bernoulli_float(prob: T, constant_time: bool) -> bool:
    if prob == 1:  # |\label{line:1check}|
        return True

    # prepare for sampling first heads index by coin flipping
    max_coin_flips = usize.exact_int_cast(T.EXPONENT_BIAS) + usize.exact_int_cast(
        T.MANTISSA_BITS
    )  # |\label{line:maxcoinflips}|

    # find number of bits to sample, rounding up to nearest byte (smallest sample size)
    buffer_len = max_coin_flips.inf_div(8)  # |\label{line:bufferlen}|

    # repeatedly flip fair coin and identify 0-based index of first heads
    first_heads_index = sample_geometric_buffer(  # |\label{line:sampling}|
        buffer_len, constant_time
    )

    # if no events occurred, return early
    if first_heads_index is None:  # |\label{line:noones}|
        return False

    # find number of zeroes in binary rep. of prob
    leading_zeroes = (
        T.EXPONENT_BIAS - 1 - prob.raw_exponent()
    )  # |\label{line:leadingzeroes}|

    # case 1: index into the leading zeroes
    if first_heads_index < leading_zeros:  # |\label{line:case1}|
        return False

    # case 2: index into implicit bit directly to left of mantissa
    if first_heads_index == leading_zeroes:  # |\label{line:case2}|
        return prob.raw_exponent() != 0

    # case 3: index into out-of-bounds/implicitly-zero bits
    if first_heads_index > leading_zeroes + T.MANTISSA_BITS:  # |\label{line:case3}|
        return False

    # case 4: index into mantissa |\label{line:case4}|
    mask = 1 << (T.MANTISSA_BITS + leading_zeroes - first_heads_index)
    return (prob.to_bits() & mask) != 0
