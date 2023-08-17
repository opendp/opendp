import opendp.prelude as dp

dp.enable_features("contrib", "honest-but-curious")


def make_duplicate(multiplicity, raises=False):
    """An example user-defined transformation from Python"""
    def function(arg):
        if raises:
            raise ValueError("test that error propagates")
        return [elem + 1 for elem in arg] * multiplicity

    def stability_map(d_in):
        return d_in * multiplicity

    return dp.t.make_user_transformation(
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        function,
        stability_map
    )

def test_make_user_transformation():
    input_domain = dp.vector_domain(dp.atom_domain(T=str))
    input_metric = dp.symmetric_distance()
    trans = (
        (input_domain, input_metric)
        >> dp.t.then_cast_default(TOA=int)
        >> make_duplicate(2)
        >> dp.t.then_clamp((1, 2))
        >> dp.t.then_sum()
        >> dp.m.then_base_discrete_laplace(1.0)
    )

    print(trans(["0", "1", "2", "3"]))
    print(trans.map(1))


def test_make_custom_transformation_error():
    import pytest
    with pytest.raises(Exception):
        make_duplicate(2, raises=True)([1, 2, 3])


def make_constant_mechanism(constant):
    """An example user-defined measurement from Python"""
    def function(_arg):
        return constant

    def privacy_map(_d_in):
        return 0.

    return dp.m.make_user_measurement(
        dp.atom_domain(T=int),
        dp.absolute_distance(int),
        dp.max_divergence(float),
        function,
        privacy_map,
        TO=dp.RuntimeType.infer(constant),
    )

def test_make_user_measurement():
    mech = make_constant_mechanism(23)
    assert mech(1) == 23
    assert mech.map(200) == 0.


def make_postprocess_frac():
    """An example user-defined postprocessor from Python"""
    def function(arg):
        return arg[0] / arg[1]

    return dp.new_function(function, float)

def test_make_user_postprocessor():
    mech = make_postprocess_frac()
    print(mech([12., 100.]))


def test_user_constructors():
    trans = dp.t.make_user_transformation(
        dp.atom_domain((2, 10)),
        dp.symmetric_distance(),
        dp.vector_domain(dp.atom_domain((2, 10)), 10),
        dp.symmetric_distance(),
        lambda x: [x] * 10,
        lambda d_in: d_in * 10
    )
    print(trans(2))
    print(trans.map(1))

    meas = dp.m.make_user_measurement(
        dp.atom_domain((2, 10)),
        dp.symmetric_distance(),
        dp.max_divergence(dp.f64),
        lambda x: [x] * 10,
        lambda d_in: float(d_in * 10),
        TO=dp.Vec[int],
    )
    print(meas(2))
    print(meas.map(1))

    post = dp.new_function(lambda x: x[0], dp.i32)

    print((meas >> post)(2))
