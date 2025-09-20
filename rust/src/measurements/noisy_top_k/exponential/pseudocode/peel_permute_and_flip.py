# type: ignore
def peel_permute_and_flip(x: list[RBig], scale: RBig, k: usize):
    natural_order = []
    sorted_order = BTreeSet.new()

    for _ in range(min(k, x.len())):
        index = permute_and_flip(x, scale)  # `\label{permute-and-flip}`
        x.remove(index)  # `\label{remove-score}`

        # map index on modified x back to original x (postprocessing)
        for del_ in sorted_order:  # `|\label{postprocess}`
            if del_ <= index:
                index += 1
            else:
                break

        sorted_order.insert(index)
        natural_order.push(index)

    return natural_order
