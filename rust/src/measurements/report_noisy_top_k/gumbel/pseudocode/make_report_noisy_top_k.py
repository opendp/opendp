# type: ignore
def make_report_noisy_top_k(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: RangeDistance[TIA],
    privacy_measure: MO,
    k: int,
    scale: f64,
    optimize: Literal["max", "min"],
) -> Measurement:
    if input_domain.element_domain.nullable:  # |\label{check-non-null}|
        raise ValueError("input domain must be non-nullable")

    if input_domain.size is not None:
        if k > input_domain.size:
            raise ValueError("k must not exceed the number of candidates")

    if k > 1 and not MO.ONE_SHOT:  # |\label{check-one-shot}|
        raise ValueError("privacy measure must support one-shot")
    
    if scale.is_sign_negative():  # |\label{check-non-negative-scale}|
        raise ValueError("scale must be non-negative")

    f_scale = FBig.try_from(scale)  # |\label{check-finite-scale}|

    if f_scale.is_zero():
        # ZERO SCALE |\label{fn-zero-scale}|
        function = Function.new_fallible(function_report_top_k(k, optimize))

    else:
        # NON-ZERO SCALE |\label{fn-nonzero-scale}|
        function = Function.new_fallible(
            function_report_noisy_top_k(k, f_scale, optimize)
        )

    def privacy_map(d_in: TIA):  # |\label{fn-privacy-map}|
        # convert to range distance
        # will multiply by 2 if not monotonic
        d_in = input_metric.range_distance(d_in) # |\label{fn-convert-to-range}|

        d_in = f64.inf_cast(d_in)  # |\label{fn-inf-cast}|

        # |\label{fn-privacy-map-call}|
        return privacy_measure.privacy_map(d_in, scale, k)

    return Measurement.new(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_measure=privacy_measure,
        privacy_map=privacy_map,
    )
