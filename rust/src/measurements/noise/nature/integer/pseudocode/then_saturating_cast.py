# type: ignore
def then_saturating_cast(modular: bool) -> Function[Vec[IBig], Vec[TO]]:
    return Function.new(
        lambda x: [
            TO.saturating_cast(x_i + IBig.from_(TO.MIN_FINITE)) if modular else TO.saturating_cast(x_i)
            for x_i in x
        ]
    )
