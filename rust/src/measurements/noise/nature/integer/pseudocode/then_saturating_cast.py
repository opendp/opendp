# type: ignore
def then_saturating_cast() -> Function[Vec[IBig], Vec[TO]]:
    return Function.new(lambda x: [TO.saturating_cast(x_i) for x_i in x])
