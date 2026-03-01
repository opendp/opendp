# type: ignore
def score_candidates(
    x: Iterator[TIA], 
    candidates: list[TIA], 
    alpha_num: u64,
    alpha_den: u64,
    size_limit: u64
) -> Iterator[usize]:
    # count of the number of records between...
    #  (-inf, c1), [c1, c2), [c2, c3), ..., [ck, inf)
    hist_ro = [0] * candidates.len() + 1 # histogram of right-open intervals
    #  (-inf, c1], (c1, c2], (c2, c3], ..., (ck, inf)
    hist_lo = [0] * candidates.len() + 1 # histogram of left-open intervals

    for x_i in x:
        idx_lt = candidates.partition_point(lambda c: c < x_i)
        hist_lo[idx_lt] += 1  # `\label{hist-lo-increment}`

        idx_eq = idx_lt + candidates[idx_lt:].partition_point(lambda c: c == x_i)
        hist_ro[idx_eq] += 1  # `\label{hist-ro-increment}`

    n: u64 = hist_lo.iter().sum()  # `\label{n-sum}`

    # don't care about the number of elements greater than all candidates
    hist_ro.pop()  # `\label{pop-ro}`
    hist_lo.pop()  # `\label{pop-lo}`

    lt, le = 0, 0
    for ro, lo in zip(hist_ro, hist_lo):  # `\label{zip-hist}`
        # cumsum the right-open histogram to get the total number of records less than the candidate
        lt += ro  # `\label{lt-cumsum}`
        # cumsum the right-open histogram to get the total number of records lt or equal to the candidate
        le += lo  # `\label{le-cumsum}`

        gt = n - le # `\label{gt}`

        # the number of records equal to the candidate is the difference between the two cumsums
        lt_lim, gt_lim = lt.min(size_limit), gt.min(size_limit)  # `\label{lt-gt-lim}`

        # a_den * |    (1 - a)         * #(x < c)    -            a * #(x > c)|
        yield ((alpha_den - alpha_num) * lt_lim).abs_diff(alpha_num * gt_lim)  # `\label{score}`
