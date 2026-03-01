# type: ignore
def make_int_to_bigint(
    input_space: tuple[VectorDomain[AtomDomain[T]], LpDistance[P, QI]],
) -> Transformation[
    VectorDomain[AtomDomain[T]],
    VectorDomain[AtomDomain[IBig]],
    LpDistance[P, QI],
    LpDistance[P, RBig],
]:
    input_domain, input_metric = input_space

    def stability_map(d_in):
        try:
            return RBig.try_from(d_in)
        except Exception:
            raise f"d_in ({d_in}) must be finite"

    return Transformation.new(
        input_domain,
        VectorDomain(
            element_domain=AtomDomain.default(IBig),
            size=input_domain.size,
        ),
        Function.new(lambda x: [IBig.from_(x_i) for x_i in x]),
        input_metric,
        LpDistance.default(),
        StabilityMap.new_fallible(stability_map),
    )
