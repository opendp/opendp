from opendp.transformations import partial_cast_default, partial_clamp, make_bounded_sum
from opendp.measurements import make_base_discrete_laplace
from opendp.combinators import *
from opendp.mod import enable_features

from opendp.domains import vector_domain, atom_domain
from opendp.metrics import symmetric_distance, absolute_distance
from opendp.measures import max_divergence
from opendp.typing import *

enable_features("contrib", "honest-but-curious")


def make_duplicate(multiplicity, raises=False):
    """An example user-defined transformation from Python"""
    def function(arg):
        if raises:
            raise ValueError("test that error propagates")
        return [elem + 1 for elem in arg] * multiplicity

    def stability_map(d_in):
        return d_in * multiplicity

    return make_user_transformation(
        vector_domain(atom_domain(T=int)),
        vector_domain(atom_domain(T=int)),
        function,
        symmetric_distance(),
        symmetric_distance(),
        stability_map
    )

def test_make_user_transformation():
    input_domain = vector_domain(atom_domain(T=str))
    input_metric = symmetric_distance()
    trans = (
        (input_domain, input_metric)
        >> partial_cast_default(TOA=int)
        >> make_duplicate(2)
        >> partial_clamp((1, 2))
        >> make_bounded_sum((1, 2))
        >> make_base_discrete_laplace(1.0)
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

    def stability_map(_d_in):
        return 0.

    return make_user_measurement(
        atom_domain(T=int),
        function,
        absolute_distance(int),
        max_divergence(float),
        stability_map,
        int,
    )

def test_make_user_measurement():
    mech = make_constant_mechanism(23)
    print(mech(1))

    assert mech.map(200) == 0.
    

def make_postprocess_frac():
    """An example user-defined postprocessor from Python"""
    def function(arg):
        return arg[0] / arg[1]

    return make_user_postprocessor(function, float)

def test_make_user_postprocessor():
    mech = make_postprocess_frac()
    print(mech([12., 100.]))


def test_user_constructors():

    from opendp.combinators import make_user_transformation, make_user_measurement
    from opendp.domains import vector_domain, atom_domain
    from opendp.metrics import symmetric_distance
    from opendp.measures import max_divergence

    trans = make_user_transformation(
        atom_domain((2, 10)),
        vector_domain(atom_domain((2, 10)), 10),
        lambda x: [x] * 10,
        symmetric_distance(),
        symmetric_distance(),
        lambda d_in: d_in * 10
    )
    print(trans(2))
    print(trans.map(1))


    meas = make_user_measurement(
        atom_domain((2, 10)),
        lambda x: [x] * 10,
        symmetric_distance(),
        max_divergence(f64),
        lambda d_in: float(d_in * 10),
        Vec[int],
    )
    print(meas(2))
    print(meas.map(1))


    post = make_user_postprocessor(lambda x: x[0], i32)

    print((meas >> post)(2))
