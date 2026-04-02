import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib")


# make-base-constant
def make_base_constant(constant):
    """Construct a Measurement that only returns a constant value."""

    def function(_arg: int):
        return constant

    def privacy_map(d_in: int) -> float:
        return 0.0

    return dp.m.make_user_measurement(
        input_domain=dp.atom_domain(T=int),
        input_metric=dp.absolute_distance(T=int),
        output_measure=dp.max_divergence(),
        function=function,
        privacy_map=privacy_map,
        TO=type(constant),
    )


# /make-base-constant


# use-measurement
meas = (
    (
        dp.vector_domain(dp.atom_domain((0, 10))),
        dp.symmetric_distance(),
    )
    >> dp.t.then_sum()
    >> make_base_constant("denied")
)
meas([2, 3, 4])
meas.map(1)
# /use-measurement
