# type: ignore
def make_laplace_expr(
    input_domain: ExprDomain[C], 
    input_metric: L1Distance[TIA], 
    scale: f64,
    k: i32, 
) -> Measurement:

    # Verify input
    scale.assert_non_negative()
    assert_equal(input_domain.active_series.field.dtype, f64)

    laplace_measurement = make_base_laplace(
        input_domain,
        input_metric,
        scale,
        k
    )

    def function(frame, expr: Expr) -> Expr:
        def lap_function(s):
            vec = s.to_vec()
            noisy_vec = laplace_measurement(vec)
            return serie(s.name, noisy_vec)
        expr.map(lambda x: lap_function(x), frame.active_series)
        return expr

    privacy_map = laplace_measurement.privacy_map

    return Measurement(
        input_domain=input_domain,
        function=function,
        input_metric=input_metric,
        output_measure=MaxDivergence,
        stability_map=stability_map,
    )
