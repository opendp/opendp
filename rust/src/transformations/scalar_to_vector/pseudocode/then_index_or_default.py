# type: ignore
def then_index_or_default(
    index: usize,
) -> Function[Vec[T], T]:
    return Function.new(lambda x: x[index] if index < len(x) else T.default())
