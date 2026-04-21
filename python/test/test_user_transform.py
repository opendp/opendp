import opendp.prelude as dp
import pytest



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
        >> dp.m.then_laplace(1.0)
    )

    print('trans(["0", "1", "2", "3"])', trans(["0", "1", "2", "3"]))
    print("trans.map(1)", trans.map(1))


def test_make_custom_transformation_error():
    with pytest.raises(Exception):
        make_duplicate(2, raises=True)([1, 2, 3])


def test_numpy_user_transformation_roundtrip():
    np = pytest.importorskip("numpy")

    input_is_ndarray = False

    def function(arg):
        nonlocal input_is_ndarray
        input_is_ndarray |= isinstance(arg, np.ndarray)
        return [v + 1 for v in arg]

    trans = dp.t.make_user_transformation(
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        dp.vector_domain(dp.atom_domain(T=int)),
        dp.symmetric_distance(),
        function,
        lambda d_in: d_in,
    )

    result = dp.as_array()(trans(np.array([1, 2, 3], dtype=np.int32)))

    assert not input_is_ndarray
    assert isinstance(result, np.ndarray)
    assert np.array_equal(result, np.array([2, 3, 4], dtype=np.int32))


def make_constant_mechanism(constant):
    """An example user-defined measurement from Python"""
    def function(_arg):
        return constant

    def privacy_map(_d_in):
        return 0.

    return dp.m.make_user_measurement(
        dp.atom_domain(T=int),
        dp.absolute_distance(int),
        dp.max_divergence(),
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
    assert mech([12., 100.]) == 0.12


def test_user_constructors():
    np = pytest.importorskip("numpy")

    trans = dp.t.make_user_transformation(
        dp.atom_domain((2, 10)),
        dp.symmetric_distance(),
        dp.vector_domain(dp.atom_domain((2, 10)), 10),
        dp.symmetric_distance(),
        lambda x: [x] * 10,
        lambda d_in: d_in * 10
    )
    assert trans(2) == [2] * 10
    assert np.array_equal(dp.as_array()(trans(2)), np.repeat(2, 10))
    assert trans.map(1) == 10

    meas = dp.m.make_user_measurement(
        dp.atom_domain((2, 10)),
        dp.symmetric_distance(),
        dp.max_divergence(),
        lambda x: [x] * 10,
        lambda d_in: float(d_in * 10),
        TO=dp.Vec[int],
    )

    assert meas(2) == [2] * 10
    assert np.array_equal((meas >> dp.as_array())(2), np.repeat(2, 10))
    assert meas.map(1) == 10

    assert (meas >> (lambda x: x[0]))(2) == 2

def test_hash():
    def get_elements(mechanisms):
        # ensure that all mechanisms have homogeneous...
        input_domain, = {m.input_domain for m in mechanisms} # ...input domain,
        input_metric, = {m.input_metric for m in mechanisms} # ...input metric,
        output_measure, = {m.output_measure for m in mechanisms} # ...and measure

        return input_domain, input_metric, output_measure
    
    def make_mock_basic_composition(mechanisms):
        input_domain, input_metric, output_measure = get_elements(mechanisms)

        # ensure that the privacy measure is pure-DP
        assert output_measure == dp.max_divergence()

        return dp.m.make_user_measurement(
            input_domain, input_metric, output_measure,
            function=lambda arg: [M(arg) for M in mechanisms], 
            privacy_map=lambda d_in: sum(M.map(d_in) for M in mechanisms))
    
    make_mock_basic_composition([
        dp.m.make_randomized_response_bool(.75)
    ] * 3)

    with pytest.raises(ValueError):
        make_mock_basic_composition([
            dp.m.make_randomized_response_bool(.75),
            dp.m.make_gaussian(dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float), 1.)
        ])
