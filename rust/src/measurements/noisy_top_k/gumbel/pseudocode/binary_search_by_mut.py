# type: ignore
def binary_search_by_mut(
    x: list[T],
    f: Callable[[T], Literal["less", "greater"]],
) -> int:
    size = x.len()
    if size == 0:  # |\label{fn-zero-size}|
        return 0
    
    base = 0 # |\label{fn-base}|

    while size > 1:
        half = size // 2
        mid = base + half

        cmp = f(x[mid])
        base = base if "greater" == cmp else mid

        size -= half

    cmp = f(x[base])
    return base + int(cmp == "less")
