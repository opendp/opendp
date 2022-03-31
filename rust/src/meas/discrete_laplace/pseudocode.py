# type: ignore
def make_base_discrete_laplace(scale: TA, granularity: TA, bounds: Option[(TA, TA)], D, MI):
    """
    :param scale: noise scale
    :param granularity: lower bound on the distance between adjacent outputs
    :param bounds: if set, algorithm runs in constant-time
    :param D: type of input domain (defines dimensionality of input arguments)
    :param MI: type if input metric (AbsoluteDistance if D is scalar, otherwise L1Distance)
    """
    assert carrier(D) == T  # Members of D have type T
    assert atom(D) == TA  # The atomic type is TA
    assert distance(MI) == TA  # Distances are also of type TA

    # Where $k = \lceil\log_2(granularity)\rceil$, $c = 2^{-k}$
    # Transform input argument with a c-lipschitz extension
    c: TA = 2 ** -ceil(log2(granularity))
    scale = scale * c
    if bounds:
        bounds = round_cast(round(bounds * c), (i64, i64))

    def function(arg: T) -> T:
        # operations are vectorized if T is a collection
        shift = round_cast(round(arg * c), i64) # $\lfloor arg \cdot c \rceil_{i64}$
        shift = round_cast(two_sided_geometric(shift, scale, bounds), T)
        # postprocess
        return shift / c

    def privacy_relation(d_in: TA) -> TA:
        # accounts for increase in sensitivity from rounding
        return (d_in + granularity) / scale

    return Transformation(
        input_domain=D.default(),
        output_domain=D.default(),
        function=function,
        input_metric=MI.default(),
        output_metric=MaxDivergence.default(),
        privacy_relation=privacy_relation,
    )
