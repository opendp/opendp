# type: ignore
def counting_query_stability_map(
    invariant: Invariant | None,
) -> StabilityMap[L01InfDistance[M], LpDistance[P, f64]]:
    if invariant == "Lengths":
        return StabilityMap.new(lambda _: 0.0)

    def stability_map(d_in: tuple[u32, u32, u32]) -> f64:
        return l01inf_to_lp(d_in, P)

    return StabilityMap.new_fallible(stability_map)
