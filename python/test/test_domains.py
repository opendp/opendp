import opendp.prelude as dp
import pytest


def test_atom_domain():
    atom_domain = dp.atom_domain(T=int, bounds=(1, 2))
    assert atom_domain == "AtomDomain(bounds=[1, 2], T=i32)"
    assert atom_domain.carrier_type == dp.i32
    assert atom_domain.bounds == (1, 2)

    atom_domain = dp.atom_domain(T=float, nullable=True)
    assert atom_domain.nullable


def test_option_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nullable=True)
    option_domain = dp.option_domain(atom_domain)
    assert option_domain.element_domain == atom_domain


def test_series_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nullable=True)
    series_domain = dp.series_domain("A", atom_domain)
    assert series_domain.name == "A"
    assert series_domain.element_domain == atom_domain
    assert not series_domain.nullable


def test_lazyframe_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nullable=True)
    series_domain = dp.series_domain("A", atom_domain)
    frame_domain = dp.with_margin(
        dp.lazyframe_domain([series_domain]),
        by=["A"],
        max_influenced_partitions=10,
        max_partition_contributions=2,
        max_num_partitions=20,
        max_partition_length=1000,
        public_info="keys",
    )
    assert frame_domain.series_domains == [series_domain]

    assert frame_domain.get_margin([]) == dp.polars.Margin(
        max_influenced_partitions=1, max_num_partitions=1, public_info="keys"
    )

    assert frame_domain.get_margin(["A"]) == dp.polars.Margin(
        max_influenced_partitions=10,
        max_partition_contributions=2,
        max_num_partitions=20,
        max_partition_length=1000,
        public_info="keys",
    )

    assert frame_domain.get_margin(["A", "B"]) == dp.polars.Margin(
        max_partition_contributions=2,
        max_partition_length=1000,
    )

    with pytest.raises(ValueError, match="to be a list of strings"):
        frame_domain.get_margin("A")


def test_vector_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nullable=True)
    vector_domain = dp.vector_domain(atom_domain, size=10)
    assert vector_domain.size == 10
    assert vector_domain.element_domain == atom_domain
