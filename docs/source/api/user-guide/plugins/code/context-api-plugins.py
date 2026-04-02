import opendp.prelude as dp

dp.enable_features("honest-but-curious", "contrib")


# register-anything-constant
def make_anything_constant(input_domain, input_metric, constant):
    return dp.m.make_user_measurement(
        input_domain=input_domain,
        input_metric=input_metric,
        output_measure=dp.max_divergence(),
        function=lambda _: constant,
        privacy_map=lambda _: 0.0,
    )


dp.register(make_anything_constant)
context = dp.Context.compositor(
    data=[1, 2, 3],
    privacy_unit=dp.unit_of(contributions=36),
    privacy_loss=dp.loss_of(epsilon=1.0),
    split_evenly_over=2,
)
context.query().anything_constant("denied").release()
# /register-anything-constant


# register-int-constant
def make_int_constant(constant):
    return dp.m.make_user_measurement(
        input_domain=dp.atom_domain(T=int),
        input_metric=dp.absolute_distance(T=int),
        output_measure=dp.max_divergence(),
        function=lambda _: constant,
        privacy_map=lambda _: 0.0,
    )


dp.register(make_int_constant)
context.query().clamp((0, 5)).sum().int_constant("denied").release()
# /register-int-constant
