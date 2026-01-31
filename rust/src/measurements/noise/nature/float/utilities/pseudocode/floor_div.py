# type: ignore
def floor_div(a: IBig, b: UBig) -> IBig:
    if Sign.Positive == a.sign():
        return a / b
    else:
        return (a - b + 1) / b
