# type: ignore
def make_sequential_odometer(
    input_domain: DI,
    input_metric: MI,
    output_measure: MO,
) -> Odometer[DI, MI, MO, Measurement[DI, TO, MI, MO], TO]:
    def function(arg: DI_Carrier, wrapper: Wrapper | None):
        return new_sequential_odometer_queryable(
            input_domain,
            input_metric,
            output_measure,
            arg,
            wrapper)
    
    return Odometer.new(
        input_domain,
        Function.new_interactive(function),
        input_metric,
        output_measure)
