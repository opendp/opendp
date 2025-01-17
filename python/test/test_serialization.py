import pytest

import opendp.prelude as dp


@pytest.mark.parametrize(
    "_,dp_obj",  # Unused parameter gives us a readable test name.
    [
        (str(obj), obj)
        for obj in [
            dp.atom_domain(bounds=(0, 10)),
            dp.categorical_domain(['A', 'B', 'C']),
            dp.series_domain('A', dp.atom_domain(bounds=(0, 10))),
            dp.lazyframe_domain([dp.series_domain('A', dp.atom_domain(bounds=(0, 10)))]),
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
        ]
    ],
)
def test_not_serializable(_, dp_obj):
    with pytest.raises(Exception, match="OpenDP JSON Encoder does not handle"):
        dp.serialize(dp_obj)
