# type: ignore
def counting_query_stability_map(
    public_info: Literal["Keys"] | Literal["Lengths"] | None,
) -> StabilityMap[PartitionDistance[M], LpDistance[P, f64]]:

    if public_info == "Lengths":  # `\label{public-info}`
        return StabilityMap.new(lambda _: 0.)
        
    def norm_map(d_in: f64) -> f64:  # `\label{norm-map}`
        if P == 1:
            return d_in
        if P == 2:
            return d_in.inf_sqrt()
        raise ValueError("unsupported Lp norm. Must be an L1 or L2 norm.")
    
    def stability_map(d_in: tuple[u32, u32, u32]) -> f64:
        l0, l1, l_inf = d_in  # `\label{l01i}`
        l0_p = norm_map(f64.from_(l0))  # `\label{l0-p}`
        l1_p = f64.from_(l1)
        l_inf_p = f64.from_(l_inf)
        return l1_p.total_min(l0_p.inf_mul(l_inf_p))  # `\label{final-bound-impl}`

    return StabilityMap.new_fallible(stability_map)  # `\label{stability-map}`
