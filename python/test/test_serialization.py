import opendp.prelude as dp


def test_atom_domain_serialization():
    domain = dp.atom_domain(bounds=(0, 10))
    serialized = dp.serialize(domain)
    deserialized = dp.deserialize(serialized)
    assert domain == deserialized