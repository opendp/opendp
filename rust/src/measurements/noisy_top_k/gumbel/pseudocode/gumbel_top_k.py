# type: ignore
def gumbel_top_k(
    x: list[TIA], k: usize, scale: FBig, negate: bool,
) -> list[usize]:
    if scale.is_zero():
        if negate:  # |\label{fn-optimize}|
            cmp = lambda a, b: a < b
        else:
            cmp = lambda a, b: a > b
        
        def max_sample(a, b):  # |\label{fn-max-sample-exact}|
            return a if cmp(a[1], b[1]) else b

        return [i for i, _ in top(x, k, max_sample)]   # |\label{fn-top-exact}|

    if all(w[0] == w[1] for w in windows(x, 2)):
        # All values are the same.  # |\label{fn-all-same}|
        x.shuffle()
        return x[:k]

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
    y = ((i, -x_i if negate else x_i) for i, x_i in x)

    # Initialize partial sample.  # |\label{fn-init-sample}|
    def partial_sample(shift):
        rv = Gumbel(shift, scale)  # |\label{fn-rv}|
        return PartialSample.new(rv)  # |\label{fn-partial-sample}|

    y = ((i, partial_sample(y_i)) for i, y_i in y)

    # Reduce to the k pairs with largest samples.  # |\label{fn-max-sample}|
    def max_sample(a, b):
        return a if a[1].greater_than(b[1]) else b

    y_top = top(y, k, max_sample) # |\label{fn-top}|

    # Discard samples, keep indices.  # |\label{fn-return-indices}|
    return [i for i, _ in y_top]
