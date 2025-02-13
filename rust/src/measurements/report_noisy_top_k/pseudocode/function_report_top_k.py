# type: ignore
def function_report_top_k(k: int, optimize: str) -> Callable[[list[TIA]], list[int]]:
    def function(x: list[TIA]) -> int:
        if optimize == "max":  # |\label{fn-optimize}|
            cmp = lambda l, r: l > r
        else:
            cmp = lambda l, r: l < r

        def max_sample(l, r):  # |\label{fn-max-sample}|
            return l if cmp(l[1], r[1]) else r

        return [i for i, _ in top(x, k, max_sample)]   # |\label{fn-top}|

    return function
