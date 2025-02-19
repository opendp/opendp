# type: ignore
def function_report_top_k(k: int, optimize: str) -> Callable[[list[TIA]], list[int]]:
    def function(x: list[TIA]) -> int:
        if optimize == "max":  # |\label{fn-optimize}|
            cmp = lambda a, b: a > b
        else:
            cmp = lambda a, b: a < b

        def max_sample(a, b):  # |\label{fn-max-sample}|
            return a if cmp(a[1], b[1]) else b

        return [i for i, _ in top(x, k, max_sample)]   # |\label{fn-top}|

    return function
