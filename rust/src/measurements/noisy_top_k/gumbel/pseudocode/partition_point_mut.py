# type: ignore
def partition_point_mut(
    x: list[T],
    pred: Callable[[T], bool],
) -> usize:
    return binary_search_by_mut(x, lambda x_i: "less" if pred(x_i) else "greater")
