# type: ignore
def make_noisy_top_k(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: LInfDistance[TIA],
    output_measure: MO,
    k: usize,
    scale: f64,
    negate: bool,
    distribution: str | None = None,
) -> Measurement:
    if input_domain.element_domain.nan():  # |\label{check-non-nan}|
        raise "input domain elements must be non-nan"

    if input_domain.size is not None:
        if k > input_domain.size:
            raise "k must not exceed the number of candidates"

    if (
        not scale.is_finite() or scale.is_sign_negative()
    ):  # |\label{check-non-negative-scale}|
        raise "scale must be finite and non-negative"

    monotonic = input_metric.monotonic

    def privacy_map(d_in: TIA):  # |\label{fn-privacy-map}|
        # convert to range distance
        d_in = d_in if monotonic else d_in.inf_add(d_in)
        d_in = f64.inf_cast(d_in)  # |\label{fn-inf-cast}|

        if d_in.is_sign_negative():  # |\label{din-non-neg}|
            raise "sensitivity must be non-negative"

        if d_in.is_zero():  # |\label{din-non-zero}|
            return 0.0

        if scale.is_zero():  # |\label{scale-zero}|
            return f64.INFINITY

        # |\label{fn-privacy-map-call}|
        return MO.privacy_map(distribution, d_in, scale, k)

    return Measurement.new(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=output_measure,
        function=lambda x: noisy_top_k(x, scale, k, negate, distribution == "gumbel"),
        privacy_map=privacy_map,
    )
