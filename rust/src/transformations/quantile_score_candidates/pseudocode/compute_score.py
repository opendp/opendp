# type: ignore
def compute_score(
    x: list[TIA], 
    candidates: list[TIA], 
    alpha_num: usize,
    alpha_den: usize,
    size_limit: usize
) -> list[usize]:

    x = list(sorted(x))

    num_lt = [0] * len(candidates)
    num_eq = [0] * len(candidates)

    count_lt_eq_recursive(
        num_lt, # mutated in-place
        num_eq, # mutated in-place
        edges=candidates,
        x=x,
        x_start_idx=0)

    def score(lt, eq):
        return abs_diff(
            alpha_den * min(lt, size_limit),
            alpha_num * min(len(x) - eq, size_limit))

    return [score(lt, eq) for lt, eq in zip(num_lt, num_eq)]
