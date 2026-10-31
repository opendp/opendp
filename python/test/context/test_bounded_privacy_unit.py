import pytest
import opendp.prelude as dp


def _vector(size):
    return dp.vector_domain(dp.atom_domain(T=float, nan=False), size=size)


@pytest.mark.parametrize("contributions", [1, 3])
@pytest.mark.parametrize("ordered", [False, True])
@pytest.mark.parametrize("splitting", [{"split_evenly_over": 1}, {}])
def test_sized_vector_rejects_odd_contributions(contributions, ordered, splitting):
    with pytest.raises(ValueError, match="Contribution bounds must be even"):
        dp.Context.compositor(
            data=[1.0, 2.0, 3.0],
            domain=_vector(3),
            privacy_unit=dp.unit_of(contributions=contributions, ordered=ordered),
            privacy_loss=dp.loss_of(epsilon=1.0),
            **splitting,
        )


def test_sized_vector_rejects_odd_contributions_before_reading_data():
    with pytest.raises(ValueError, match="Contribution bounds must be even"):
        dp.Context.compositor(
            data=object(),
            domain=_vector(3),
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )


@pytest.mark.parametrize("ordered", [False, True])
def test_sized_vector_accepts_even_contributions_and_changes(ordered):
    for privacy_unit in [
        dp.unit_of(contributions=2, ordered=ordered),
        dp.unit_of(changes=1, ordered=ordered),
    ]:
        context = dp.Context.compositor(
            data=[1.0, 2.0, 3.0],
            domain=_vector(3),
            privacy_unit=privacy_unit,
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )
        assert context.d_in == privacy_unit[1]
        assert context.accountant.input_metric == privacy_unit[0]
        if privacy_unit[0] == dp.symmetric_distance():
            assert isinstance(context.query().count().laplace().release(), int)


def test_unknown_vector_size_accepts_odd_contributions():
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0],
        domain=_vector(None),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert isinstance(context.query().count().laplace().release(), int)


@pytest.mark.parametrize("array_domain", ["array2", "arrayd"])
def test_fixed_numpy_shape_rejects_odd_contributions(array_domain):
    np = pytest.importorskip("numpy")
    domain = (
        dp.numpy.array2_domain(size=3, num_columns=2, T=float)
        if array_domain == "array2"
        else dp.numpy.arrayd_domain(shape=(3, 2), T=float)
    )
    with pytest.raises(ValueError, match="Contribution bounds must be even"):
        dp.Context.compositor(
            data=np.zeros((3, 2)),
            domain=domain,
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )


def test_numpy_integer_contribution_bound_is_checked():
    np = pytest.importorskip("numpy")
    with pytest.raises(ValueError, match="Contribution bounds must be even"):
        dp.Context.compositor(
            data=[1.0, 2.0, 3.0],
            domain=_vector(3),
            privacy_unit=dp.unit_of(contributions=np.uint32(1)),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )


def test_numpy_unknown_rows_and_even_contributions_remain_valid():
    np = pytest.importorskip("numpy")
    for size, contributions in [(None, 1), (3, 2)]:
        context = dp.Context.compositor(
            data=np.zeros((3, 2)),
            domain=dp.numpy.array2_domain(size=size, num_columns=2, T=float),
            privacy_unit=dp.unit_of(contributions=contributions),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )
        assert isinstance(context, dp.Context)


@pytest.mark.parametrize("grouping", [[], ["group"]])
def test_frame_invariant_lengths_reject_odd_contributions(grouping):
    pl = pytest.importorskip("polars")
    with pytest.raises(ValueError, match="Contribution bounds must be even"):
        dp.Context.compositor(
            data=pl.DataFrame({"x": [1, 2, 3], "group": ["a", "a", "b"]}).lazy(),
            margins=[dp.polars.Margin(by=grouping, invariant="lengths")],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
        )


@pytest.mark.parametrize(
    "margin", [dp.polars.Margin(invariant="keys"), dp.polars.Margin(max_length=3)]
)
def test_frame_without_invariant_length_accepts_odd_contributions(margin):
    pl = pytest.importorskip("polars")
    context = dp.Context.compositor(
        data=pl.DataFrame({"x": [1, 2, 3]}).lazy(),
        margins=[margin],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert isinstance(context, dp.Context)


def test_frame_invariant_length_accepts_even_contributions():
    pl = pytest.importorskip("polars")
    context = dp.Context.compositor(
        data=pl.DataFrame({"x": [1, 2, 3]}).lazy(),
        margins=[dp.polars.Margin(invariant="lengths")],
        privacy_unit=dp.unit_of(contributions=2),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert context.d_in == 2


def test_fixed_vector_aggregate_allows_odd_l1_bound():
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0],
        domain=_vector(3),
        privacy_unit=dp.unit_of(l1=1.0),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert len(context.query().laplace().release()) == 3


def test_custom_descriptor_is_not_interpreted_as_a_fixed_size():
    domain = dp.user_domain(
        identifier="list of floats; size is not an OpenDP domain constraint",
        member=lambda data: (
            isinstance(data, list) and all(isinstance(value, float) for value in data)
        ),
        descriptor={"size": 3},
    )
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0],
        domain=domain,
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    assert context.d_in == 1


@pytest.mark.parametrize("splitting", [{"split_evenly_over": 1}, {}])
def test_nested_context_rejects_odd_bound_on_sized_output(splitting):
    query = dp.Query(
        (_vector(3), dp.symmetric_distance()),
        dp.max_divergence(),
        d_in=1,
        d_out=1.0,
    )
    with pytest.raises(ValueError, match="Contribution bounds must be even"):
        query.compositor(**splitting)


def test_nested_context_uses_transformed_bound():
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    child = (
        context.query()
        .resize(size=3, constant=0.0)
        .compositor(split_evenly_over=1)
        .release()
    )
    assert child.d_in == 2
    assert isinstance(child.query().count().laplace().release(), int)


def test_lower_level_odd_distance_search_is_unchanged():
    domain = dp.vector_domain(dp.atom_domain(bounds=(0, 10), T=int), size=3)

    measurement = (
        (domain, dp.symmetric_distance())
        >> dp.t.then_sum()
        >> dp.m.then_laplace(scale=1.0)
    )
    assert measurement.map(1) == 0.0
    assert (
        dp.binary_search(
            lambda distance: measurement.check(distance, 1.0), bounds=(1, 5), T=int
        )
        == 1
    )
