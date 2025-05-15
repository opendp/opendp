# type: ignore
def make_fully_adaptive_composition(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Odometer[DI, MI, MO, Measurement[DI, TO, MI, MO], TO]:

    # check if fully adaptive composition is supported
    output_measure.theorem(Adaptivity.FullyAdaptive)

    def function(
        arg: DI_Carrier,
    ) -> OdometerQueryable[Measurement[DI, TO, MI, MO], TO, MO_Distance]:
        return new_fully_adaptive_composition_queryable(
            input_domain, input_metric, output_measure, arg
        )

    return Odometer.new(
        input_domain, input_metric, output_measure, Function.new_fallible(function)
    )
