# type: ignore
def make_bounded_range_to_pureDP(meas: Measurement) -> Measurement:
    return meas.with_map(
        meas.input_metric,
        ZeroConcentratedDivergence.default(),
        PrivacyMap.new_fallible(lambda d_in: meas.privacy_map(d_in)),
    )