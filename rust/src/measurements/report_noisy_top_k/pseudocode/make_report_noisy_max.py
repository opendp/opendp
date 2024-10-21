# type: ignore
def make_report_noisy_max(
    input_domain: VectorDomain[AtomDomain[TIA]],
    input_metric: LInfDistance[TIA],
    output_measure: MO,
    scale: f64, 
    optimize: Union[Literal["max"], Literal["min"]]
) -> Measurement:
    if input_domain.element_domain.nullable:
        raise ValueError("input domain must be non-nullable")

    if scale < 0: 
        raise ValueError("scale must be non-negative") 

    f_scale = Fraction(scale)

    def function(scores: list[TIA]):
        return select_score(scores, optimize, f_scale, MO)

    def privacy_map(d_in: TIA): 
        # convert to range distance 
        # will multiply by 2 if not monotonic 
        d_in = input_metric.range_distance(d_in) 

        d_in = f64.inf_cast(d_in) 
        if d_in < 0: 
            raise ValueError("input distance must be non-negative") 

        if d_in == 0: 
            return 0 

        return d_in.inf_div(scale) 

    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_measure=output_measure,
        privacy_map=privacy_map,
    )
