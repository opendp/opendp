# type: ignore
def function_report_noisy_top_k(
    k: int, scale: FBig, optimize: str
) -> Callable[[list[TIA]], list[int]]:
    def function(x: list[TIA]) -> int:
        def try_cast(v):
            try:
                return FBig.try_from(v)
            except Exception:
                return None

        # Cast to FBig.  # |\label{fn-cast}|
        x = ((i, try_cast(x_i)) for i, x_i in enumerate(x))
        # Discard failed casts.  # |\label{fn-filter-nan}|
        x = ((i, x_i) for i, x_i in x if x_i is not None)

        # Normalize sign.  # |\label{fn-normalize-sign}|
        y = ((i, -x_i if optimize == "min" else x_i) for i, x_i in x)

        # Initialize partial sample.  # |\label{fn-init-sample}|
        def partial_sample(shift):
            rv = MO.random_variable(shift, scale)  # |\label{fn-rv}|
            return PartialSample.new(rv)  # |\label{fn-partial-sample}|

        y = ((i, partial_sample(y_i)) for i, y_i in y)

        # Reduce to the k pairs with largest samples.  # |\label{fn-max-sample}|
        def max_sample(l, r):
            return l if l[1].greater_than(r[1]) else r

        y_top = top(y, k, max_sample) # |\label{fn-top}|

        # Discard samples, keep indices.  # |\label{fn-return-indices}|
        return [i for i, _ in y_top]

    return function
