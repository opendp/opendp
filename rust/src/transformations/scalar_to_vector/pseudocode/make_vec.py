# type: ignore
def make_vec(
    input_space: tuple[AtomDomain[T], AbsoluteDistance[Q]],
) -> Transformation[
    AtomDomain[T], VectorDomain[AtomDomain[T]], AbsoluteDistance[Q], LpDistance[P, Q]
]:
    input_domain, input_metric = input_space
    return Transformation.new(
        input_domain,
        VectorDomain.new(input_domain).with_size(1),
        lambda arg: [arg],
        input_metric,
        LpDistance.default(),
        lambda d_in: d_in,
    )
