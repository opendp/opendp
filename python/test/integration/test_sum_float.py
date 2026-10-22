import os

import pytest

from opendp.transformations import *
from opendp.mod import OpenDPException, enable_features

enable_features("contrib")

# Debug binaries only dispatch f64. Release tests also exercise f32.
FLOAT_TYPES = ["f32", "f64"] if os.environ.get("OPENDP_TEST_RELEASE") else ["f64"]
PAIRWISE_ORDERED_ERROR = (
    "Pairwise summation is not supported by ordered float sums. Use Sequential instead."
)


def test_make_bounded_float_checked_sum():
    sum_trans = make_bounded_float_checked_sum(100, (0.0, 10.0))
    assert sum_trans([1.0, 2.0, 4.0]) == 7

    make_bounded_float_checked_sum(100, (0.0, 10.0), "Pairwise<T>")([1.0, 2.0])
    make_bounded_float_checked_sum(100, (0.0, 10.0), "Pairwise<f64>")([1.0, 2.0])
    make_bounded_float_checked_sum(100, (0.0, 10.0), "Sequential<T>")([1.0, 2.0])
    make_bounded_float_checked_sum(100, (0.0, 10.0), "Sequential<f64>")([1.0, 2.0])


def test_make_sized_bounded_float_checked_sum():
    sum_trans = make_sized_bounded_float_checked_sum(100, (0.0, 10.0))
    assert sum_trans([1.0, 2.0, 4.0]) == 7
    make_sized_bounded_float_checked_sum(100, (0.0, 10.0), "Sequential<T>")([1.0, 2.0])


@pytest.mark.parametrize(
    "constructor",
    [make_bounded_float_ordered_sum, make_sized_bounded_float_ordered_sum],
)
@pytest.mark.parametrize("algorithm", [None, "Sequential<T>", "Sequential<f64>"])
def test_bounded_float_ordered_sum_default_and_sequential(constructor, algorithm):
    # This cancellation pattern distinguishes sequential from pairwise summation.
    large = float(2**54)
    args = (4, (-large, large))
    sum_trans = (
        constructor(*args) if algorithm is None else constructor(*args, algorithm)
    )

    assert sum_trans([large, 1.0, -large, 1.0]) == 1.0


@pytest.mark.parametrize("T", FLOAT_TYPES)
@pytest.mark.parametrize(
    "constructor",
    [make_bounded_float_ordered_sum, make_sized_bounded_float_ordered_sum],
)
def test_bounded_float_ordered_sum_carriers(constructor, T):
    large = float(2 ** (25 if T == "f32" else 54))
    sum_trans = constructor(4, (-large, large), f"Sequential<{T}>")

    assert sum_trans([large, 1.0, -large, 1.0]) == 1.0


@pytest.mark.parametrize("T", FLOAT_TYPES)
@pytest.mark.parametrize(
    "constructor",
    [make_bounded_float_ordered_sum, make_sized_bounded_float_ordered_sum],
)
def test_bounded_float_ordered_sum_rejects_pairwise(constructor, T):
    # Reject the unsupported algorithm during construction, before it can be invoked.
    with pytest.raises(OpenDPException) as err:
        constructor(4, (0.0, 10.0), f"Pairwise<{T}>")
    assert err.value.variant == "MakeTransformation"
    assert err.value.message == PAIRWISE_ORDERED_ERROR


@pytest.mark.parametrize(
    "constructor",
    [make_bounded_float_ordered_sum, make_sized_bounded_float_ordered_sum],
)
def test_bounded_float_ordered_sum_rejects_generic_pairwise(constructor):
    with pytest.raises(OpenDPException) as err:
        constructor(4, (0.0, 10.0), "Pairwise<T>")
    assert err.value.variant == "MakeTransformation"
    assert err.value.message == PAIRWISE_ORDERED_ERROR


@pytest.mark.parametrize("T", FLOAT_TYPES)
@pytest.mark.parametrize(
    "constructor",
    [make_bounded_float_checked_sum, make_sized_bounded_float_checked_sum],
)
def test_bounded_float_checked_sum_retains_pairwise(constructor, T):
    sum_trans = constructor(3, (0.0, 10.0), f"Pairwise<{T}>")

    assert sum_trans([1.0, 2.0, 4.0]) == 7.0
