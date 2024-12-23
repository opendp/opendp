# type: ignore
def make_bounded_range_to_zCDP(meas: Measurement) -> Measurement:
    def privacy_map(d_in: MI_Distance) -> f64:
        return meas.map(d_in).inf_powi(ibig(2)).inf_div(&8.0)

    return meas.with_map(
        meas.input_metric,
        ZeroConcentratedDivergence.default(),
        PrivacyMap.new_fallible(privacy_map),
    )