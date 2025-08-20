# type: ignore
def make_privacy_filter(
    odometer: Odometer[DI, MI, MO, Q, A],
    d_in: MI_Distance,
    d_out: MO_Distance,
) -> Measurement[DI, MI, MO, OdometerQueryable[MI, MO, Q, A]]:
    odo_function = odometer.function
    continuation_rule = new_continuation_rule(  # `\label{continuation-rule}`
        d_in, d_out, Q, A, MI_Distance, MO_Distance
    )

    def function(arg: DI_Carrier) -> OdometerQueryable[MI, MO, Q, A]:
        return wrap(continuation_rule, lambda: odo_function.eval(arg))  # `\label{eval}`

    def privacy_map(d_in_p: MI_Distance) -> MO_Distance:
        if d_in_p.total_gt(d_in):
            raise "input distance must not be greater than d_in"

        return d_out

    return Measurement.new(
        odometer.input_domain,
        odometer.input_metric,
        odometer.output_measure,
        Function.new_interactive(function),
        PrivacyMap.new_fallible(privacy_map),
    )
