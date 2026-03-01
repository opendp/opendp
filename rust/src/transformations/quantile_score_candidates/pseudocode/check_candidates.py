# type: ignore
def validate_candidates(candidates: list[T]):
    if not candidates: # `\label{empty}`
        raise ValueError("candidates must not be empty")
    
    i1 = iter(candidates)
    i2 = iter(candidates)
    next(i1)

    for c1, c2 in zip(i1, i2): # `\label{windows}`
        cmp = c1.partial_cmp(c2)
        if cmp is None or cmp != Ordering.Less:
            raise ValueError("candidates must be non-null and strictly increasing")
