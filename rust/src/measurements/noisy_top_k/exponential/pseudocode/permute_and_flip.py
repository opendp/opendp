# type: ignore
def permute_and_flip(x: list[RBig], scale: RBig):
    if scale.is_zero():  # `\label{zero-scale}`
        return max(range(x.len()), key=lambda i: x[i])

    # begin nonzero scale `\label{nonzero-scale}`
    x_max = max(x)
    permutation = list(range(x.len()))

    for left in range(x.len()):
        right = left + sample_uniform_uint_below(x.len() - left)
        permutation.swap(left, right) # fisher-yates shuffle up to left `\label{shuffle}`

        candidate = permutation[left]
        if sample_bernoulli_exp((x_max - x[candidate]) / scale):
            return candidate
    
    raise "at least one x[candidate] is equal to x_max"