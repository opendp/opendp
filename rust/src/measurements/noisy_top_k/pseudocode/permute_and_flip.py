# type: ignore
def permute_and_flip(x: list[RBig], scale: RBig, replacement: bool):
    if scale.is_zero():  # `\label{zero-scale}`
        return max(range(x.len()), key=lambda i: x[i])

    # begin nonzero scale `\label{nonzero-scale}`
    x_max = max(x)
    permutation = list(range(x.len()))

    sequence = range(0, len(x)) if replacement else repeat(0)

    for left in sequence:
        right = left + sample_uniform_uint_below(x.len() - left)
        # fisher-yates shuffle up to left `\label{shuffle}`
        permutation.swap(left, right)

        candidate = permutation[left]
        if sample_bernoulli_exp((x_max - x[candidate]) / scale):
            return candidate

    raise "at least one x[candidate] is equal to x_max"
