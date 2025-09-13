import opendp  # So we can access the private _PRIMITIVE_TYPES
from opendp._lib import import_optional_dependency
from opendp.extras.polars import Margin
import opendp.prelude as dp
import pytest


@pytest.mark.parametrize("ty", opendp.typing._PRIMITIVE_TYPES)
def test_atom_domain_primitive_types(ty):
    # Checks that all primitive types are construct-able over FFI.
    # This ensures that all Polars dtypes can be used in debug builds.
    dp.option_domain(dp.atom_domain(T=ty))


def test_atom_domain_bounds():
    atom_domain = dp.atom_domain(T=int, bounds=(1, 2))
    assert str(atom_domain) == "AtomDomain(bounds=[1, 2], T=i32)"
    assert atom_domain.carrier_type == dp.i32
    assert atom_domain.bounds == (1, 2)
    assert atom_domain != str(atom_domain)
    assert not atom_domain.nan

def test_atom_domain_nullable():
    atom_domain = dp.atom_domain(T=float, nan=True)
    assert atom_domain.carrier_type == dp.f64
    assert atom_domain.bounds is None
    assert atom_domain != str(atom_domain)
    assert atom_domain.nan


def test_option_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nan=True)
    option_domain = dp.option_domain(atom_domain)
    assert option_domain.element_domain == atom_domain


def test_series_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nan=True)
    series_domain = dp.series_domain("A", atom_domain)
    assert series_domain.name == "A"
    assert series_domain.element_domain == atom_domain
    assert not series_domain.nullable


def test_lazyframe_domain_series():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nan=True)
    series_domain = dp.series_domain("A", atom_domain)
    frame_domain = dp.lazyframe_domain([series_domain])

    assert frame_domain.get_series_domain("A") == series_domain
    assert frame_domain.columns == ["A"]


def test_lazyframe_domain_margins():
    import_optional_dependency("polars")
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nan=True)
    series_domain_a = dp.series_domain("A", atom_domain)
    series_domain_b = dp.series_domain("B", atom_domain)
    frame_domain = dp.with_margin(
        dp.lazyframe_domain([series_domain_a, series_domain_b]),
        dp.polars.Margin(
            by=["A"],
            max_groups=20,
            max_length=1000,
            invariant="keys"
        ),
    )
    assert frame_domain.get_series_domain("A") == series_domain_a

    # for coverage
    assert dp.polars.Margin(by=[]) != "not a margin"
    assert dp.polars.Margin(by=[]) != dp.polars.Margin(by=["A"])
    assert frame_domain.get_margin([]) == dp.polars.Margin(
        by=[], max_groups=1, invariant="keys"
    )

    assert frame_domain.get_margin(["A"]) == dp.polars.Margin(
        by=["A"],
        max_groups=20,
        max_length=1000,
        invariant="keys",
    )

    assert frame_domain.get_margin(["A", "B"]) == dp.polars.Margin(
        by=["A", "B"],
        max_length=1000,
    )

    # now add a margin for column B
    frame_domain = dp.with_margin(
        frame_domain,
        Margin(
            by=["B"],
            max_groups=20,
            max_length=500,
            invariant="keys"
        ),
    )

    assert frame_domain.get_margin(["A", "B"]) == dp.polars.Margin(
        by=["A", "B"],
        max_length=500,  # from B
        max_groups=400, # 20 * 20
    )

    with pytest.raises(ValueError, match="must be a sequence type; Did you mean [\"A\"]?"):
        frame_domain.get_margin("A")

    with pytest.raises(ValueError, match="must be a sequence type"):
        frame_domain.get_margin(2) # type: ignore[arg-type]


def test_vector_domain():
    atom_domain = dp.atom_domain(bounds=(1.0, 2.0), nan=True)
    vector_domain = dp.vector_domain(atom_domain, size=10)
    assert vector_domain.size == 10
    assert vector_domain.element_domain == atom_domain
