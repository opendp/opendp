# type: ignore
def make_privacy_filter(
    odometer: Odometer[DI, MI, MO, Q, A],
    d_in: MI_Distance,
    d_out: MO_Distance,
) -> Measurement[DI, OdometerQueryable[MI, MO, Q, A], MI, MO]:
    odo_function = odometer.function

    def function(arg: DI_Carrier) -> OdometerQueryable[MI, MO, Q, A]:
        # `\label{continuation-rule}`
        continuation_rule = new_continuation_rule(d_in, d_out, MI, MO)
        return wrap(continuation_rule, lambda: odo_function.eval(arg))  # `\label{eval}`

    def privacy_map(d_in_p: MI_Distance) -> MO_Distance:
        if d_in_p.total_gt(d_in):
            raise "input distance must not be greater than d_in"

        return d_out

    return Measurement.new(
        odometer.input_domain,
        Function.new_interactive(function),
        odometer.input_metric,
        odometer.output_measure,
        PrivacyMap.new_fallible(privacy_map),
    )
