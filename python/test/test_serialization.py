import pytest

import opendp.prelude as dp


@pytest.mark.parametrize(
    "_,dp_obj",  # Unused parameter gives us a readable test name.
    [
        (str(obj), obj)
        for obj in [
            dp.atom_domain(bounds=(0, 10)),
            dp.categorical_domain(['A', 'B', 'C']),
        ]
    ],
)
def test_atom_domain_serialization(_, dp_obj):
    serialized = dp.serialize(dp_obj)
    deserialized = dp.deserialize(serialized)
    assert dp_obj == deserialized
