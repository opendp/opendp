# type: ignore
def score_candidates_map(alpha_num, alpha_den, known_size) -> Callable[[int], int]:
    def stability_map(d_in):
        if known_size:
            return T.inf_cast(d_in // 2).inf_mul(alpha_den)
        else:
            abs_dist_const: u64 = max(alpha_num, alpha_den - alpha_num)  # `\label{sub}`
            return T.exact_int_cast(d_in).alerting_mul(abs_dist_const)

    return stability_map
