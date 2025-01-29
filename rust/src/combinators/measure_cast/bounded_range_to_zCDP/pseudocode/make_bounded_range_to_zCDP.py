# type: ignore
def make_bounded_range_to_zCDP(meas: Measurement) -> Measurement:
    def privacy_map(d_in: f64) -> f64:
        return meas.map(d_in).inf_powi(ibig(2)).inf_div(8.0)

    return meas.with_map( # |\label{with_map}|
        meas.input_metric,
        ZeroConcentratedDivergence,
        PrivacyMap.new_fallible(privacy_map),
    )