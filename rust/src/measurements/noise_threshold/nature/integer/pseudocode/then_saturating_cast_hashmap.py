# type: ignore
def then_saturating_cast_hashmap() -> Function[HashMap[TK, IBig], HashMap[TK, TV]]:
    return Function.new(lambda x: {k: TV.saturating_cast(v) for k, v in x.items()})
