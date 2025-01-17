import pytest

import opendp.prelude as dp


atom = dp.atom_domain(bounds=(0, 10))

@pytest.mark.parametrize(
    "_,dp_obj",  # Unused parameter gives us a readable test name.
    [
        (str(obj), obj)
        for obj in [
            # Domains:
            atom,
            dp.categorical_domain(['A', 'B', 'C']),
            dp.series_domain('A', atom),
            dp.lazyframe_domain([dp.series_domain('A', atom)]),
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
            dp.m.make_gaussian(atom, dp.absolute_distance(int), 1),
        ]
    ],
)
def test_serializable(_, dp_obj):
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    assert dp_obj == deserialized



@pytest.mark.parametrize(
    "_,dp_obj",  # Unused parameter gives us a readable test name.
    [
        (str(obj), obj)
        for obj in [
            dp.user_domain("trivial_user_domain", lambda: True),
            dp.m.new_privacy_profile(lambda x: x),
        ]
    ],
)
def test_not_serializable(_, dp_obj):
    with pytest.raises(Exception, match="OpenDP JSON Encoder does not handle"):
        dp.serialize(dp_obj)
