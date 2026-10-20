import json
import re

from opendp.extras.polars import Margin
import pytest

import opendp.prelude as dp
from opendp._lib import import_optional_dependency

atom = dp.atom_domain(bounds=(0.0, 10.0), nan=False)
input_space = dp.vector_domain(atom, size=10), dp.symmetric_distance()
chained = input_space >> dp.t.then_mean() >> dp.m.then_laplace(scale=0.5)


@pytest.mark.parametrize(
    "dp_obj",
    [
        # Python objects:
        ("nested", ("tuple", ("containing", ("domain", (atom,))))),
        {"dict key": atom},
        input_space,
        # Domains:
        atom,
        dp.categorical_domain(["A", "B", "C"]),
        dp.series_domain("A", atom),
        dp.lazyframe_domain([dp.series_domain("A", atom)]),
        dp.wild_expr_domain([]),
        # Metrics:
        dp.absolute_distance("int"),
        dp.change_one_distance(),
        dp.linf_distance("float", True),
        dp.user_distance("user_distance"),
        # Measures:
        dp.m.max_divergence(),
        dp.m.approximate(dp.m.max_divergence()),
        dp.m.user_divergence("user_divergence"),
        # Measurements:
        dp.m.make_gaussian(atom, dp.absolute_distance(float), 1),
        dp.m.then_gaussian(1),
        # Compositions:
        chained,
        dp.c.make_population_amplification(chained, population_size=100),
    ],
    ids=lambda arg: str(arg),
)
def test_serializable_equal(dp_obj):
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    # We don't want to define __eq__ just for the sake of testing,
    # so check the serializations before and after.
    # (We should remember that if the first serialization
    #  dropped some detail, this test wouldn't catch it.)
    assert serialized == dp.serialize(deserialized)


@pytest.mark.parametrize(
    "dp_obj",
    [
        dp.user_domain("trivial_user_domain", lambda: True),
        dp.m.new_privacy_profile(lambda x: x),
    ],
    ids=lambda arg: str(arg),
)
def test_not_ever_serializable(dp_obj):
    with pytest.raises(Exception, match=r"OpenDP JSON Encoder does not handle"):
        dp.serialize(dp_obj)


@pytest.mark.parametrize(
    "dp_obj",
    [
        {("tuple", "key"): "value"},
    ],
    ids=lambda arg: str(arg),
)
def test_not_json_serializable(dp_obj):
    with pytest.raises(
        Exception, match=r"keys must be str, int, float, bool or None, not tuple"
    ):
        dp.serialize(dp_obj)


def test_version_mismatch_warning():
    bad_serialized = json.dumps(
        {
            "__function__": "atom_domain",
            "__module__": "domains",
            "__kwargs__": {"bounds": {"__tuple__": [0, 10]}, "nan": False, "T": "i32"},
            "__version__": "bad-version",
        }
    )
    with pytest.warns(UserWarning, match=re.escape("(bad-version) != this version")):
        dp.deserialize(bad_serialized)
