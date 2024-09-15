# type: ignore
def make_fully_adaptive_composition(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Odometer[DI, MI, MO, Measurement[DI, TO, MI, MO], TO]:
    sequential = matches(
        output_measure.theorem(Adaptivity.FullyAdaptive),
        Sequentiality.Sequential
    )

    def function(arg: DI_Carrier):
        return new_fully_adaptive_composition_queryable(
            input_domain,
            input_metric,
            output_measure,
            arg,
            sequential)
    
    return Odometer.new(
        input_domain,
        Function.new_fallible(function),
        input_metric,
        output_measure)
