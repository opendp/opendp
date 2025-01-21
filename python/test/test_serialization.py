import pytest

import opendp.prelude as dp


atom = dp.atom_domain(bounds=(0, 10))

@pytest.mark.parametrize(
    "_readable_name,dp_obj",
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
            dp.m.then_gaussian(1),
        ]
    ],
)
def test_serializable(_readable_name, dp_obj):
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    assert dp_obj == deserialized


@pytest.mark.parametrize(
    "_readable_name,dp_obj",
    [
        (str(obj), obj)
        for obj in [
            dp.m.make_private_expr(
                dp.wild_expr_domain([], by=[]),
                dp.partition_distance(dp.symmetric_distance()),
                dp.max_divergence(),
                dp.len(scale=1.0)
            ),
        ]
    ],
)
def test_serializable_polars(_readable_name, dp_obj):
    pytest.importorskip("polars")
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    assert dp_obj == deserialized


@pytest.mark.parametrize(
    "_readable_name,dp_obj",
    [
        (str(obj), obj)
        for obj in [
            dp.user_domain("trivial_user_domain", lambda: True),
            dp.m.new_privacy_profile(lambda x: x),
        ]
    ],
)
def test_not_serializable(_readable_name, dp_obj):
    with pytest.raises(Exception, match="OpenDP JSON Encoder does not handle"):
        dp.serialize(dp_obj)
