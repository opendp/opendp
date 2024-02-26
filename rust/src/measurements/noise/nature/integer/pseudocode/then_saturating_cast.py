# type: ignore
def then_saturating_cast() -> Function[Vec[IBig], Vec[T]]:
    return Function.new(lambda x: [T.saturating_cast(x_i) for x_i in x])
