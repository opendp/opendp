# type: ignore
def top(
    iter: Iterator[T],
    k: usize,
    greater_than: Callable[[T, T], bool],
) -> list[T]:
    heap = []  # |\label{fn-heap}|
    
    if k == 0: # |\label{fn-zero}|
        return heap

    for value in iter:  # |\label{fn-iter}|
        if len(heap) == k: # |\label{fn-len}|
            if greater_than(heap[-1], value): # |\label{fn-greater}|
                continue
            heap.pop() # |\label{fn-pop}|

        # insert value into heap # |\label{fn-insert}|
        index = partition_point_mut(heap, lambda x: greater_than(x, value))
        heap.insert(index, value)
    return heap
