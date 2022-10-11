from opendp.transformations import make_cast_default, make_clamp, make_bounded_sum
from opendp.measurements import make_base_discrete_laplace
from opendp.combinators import *
from opendp.mod import enable_features
from opendp.typing import *

enable_features("contrib", "honest-but-curious")


def make_duplicate(multiplicity, raises=False):
    """An example user-defined transformation from Python"""
    def function(arg):
        if raises:
            raise ValueError("boohoohoo")
        return [elem + 1 for elem in arg] * multiplicity

    def stability_map(d_in):
        return d_in * multiplicity

    return make_default_transformation(
        function,
        stability_map,
        DI=VectorDomain[AllDomain[i32]],
        DO=VectorDomain[AllDomain[i32]],
        MI=SymmetricDistance,
        MO=SymmetricDistance,
    )

def test_make_default_transformation():
    trans = (
        make_cast_default(TIA=str, TOA=int)
        >> make_duplicate(2)
        >> make_clamp((1, 2))
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

    return make_default_measurement(
        function,
        stability_map,
        DI=AllDomain[i32],
        DO=AllDomain[i32],
        MI=AbsoluteDistance[i32],
        MO=MaxDivergence[f64],
    )

def test_make_default_measurement():
    mech = make_constant_mechanism(23)
    print(mech(1))

    assert mech.map(200) == 0.


def make_postprocess_frac():
    """An example user-defined postprocessor from Python"""
    def function(arg):
        return arg[0] / arg[1]

    return make_default_postprocessor(
        function,
        DI=VectorDomain[AllDomain[f64]],
        DO=AllDomain[f64],
    )

def test_make_default_postprocessor():
    mech = make_postprocess_frac()
    print(mech([12., 100.]))


if __name__ == "__main__":
    test_make_default_transformation()
    test_make_custom_transformation_error()
    test_make_default_measurement()
    test_make_default_postprocessor()
