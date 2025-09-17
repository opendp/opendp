# type: ignore
def get_min_k() -> i32:
    return -i32.exact_int_cast(T.EXPONENT_BIAS) - i32.exact_int_cast(T.MANTISSA_BITS) + 1