# type: ignore
def make_float_to_bigint(
    input_space: tuple[VectorDomain[AtomDomain[T]], LpDistance[P, QI]], k: i32
) -> Transformation[
    VectorDomain[AtomDomain[T]],
    VectorDomain[AtomDomain[IBig]],
    LpDistance[P, QI],
    LpDistance[P, RBig],
]:
    input_domain, input_metric = input_space
    if input_domain.element_domain.nullable():
        raise "input_domain may not contain NaN elements"

    size = input_domain.size
    rounding_distance = get_rounding_distance(k, size, T) # |\label{line:rounding-distance}|

    def elementwise_function(x_i):  # |\label{line:elementwise-function}|
        x_i = RBig.try_from(x_i).unwrap_or(RBig.ZERO)  # |\label{line:try-from}|
        return find_nearest_multiple_of_2k(x_i, k)  # |\label{line:find-nearest}|

    def stability_map(d_in):
        try:
            d_in = RBig.try_from(d_in)
        except Exception:
            raise f"d_in ({d_in}) must be finite"
        return x_mul_2k(d_in + rounding_distance, -k)  # |\label{line:x-mul-2k}|

    return Transformation.new(
        input_domain,
        VectorDomain(  # |\label{line:output-domain}|
            element_domain=AtomDomain.default(IBig),
            size=size,
        ),
        Function.new(lambda x: [elementwise_function(x_i) for x_i in x]),
        input_metric,
        LpDistance.default(),
        StabilityMap.new_fallible(stability_map),
    )
