import pytest
import opendp.prelude as dp


@pytest.mark.xfail(raises=dp.UnknownTypeException)
def test_iterable_data():
    # Currently fails with:
    #   opendp.mod.UnknownTypeException: <class 'range'>
    # Possible resolution:
    #   The data kwarg accepts iterables.
    context = dp.Context.compositor(
        data=range(100),
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    sum_query = context.query().clamp((1.0, 10.0)).sum()
    sum_query.laplace()


def test_int_data_laplace_param():
    context = dp.Context.compositor(
        data=[1, 2, 3, 4, 5],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    sum_query = context.query().clamp((1, 10)).sum()
    sum_query.laplace(100)


@pytest.mark.xfail(raises=dp.OpenDPException)
def test_mean_without_size():
    # Currently fails with:
    #   opendp.mod.OpenDPException:
    #     MakeTransformation("dataset size must be known. Either specify size in the input domain or use make_resize")
    # Possible resolution:
    #   Error message suggests fixes in terms on the new Context API.
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0, 4.0, 5.0],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    mean_query = context.query().clamp((1.0, 10.0)).mean()
    mean_query.laplace()


@pytest.mark.xfail(raises=dp.OpenDPException)
def test_int_mean():
    # Currently fails with:
    #   opendp.mod.OpenDPException:
    #     FFI("No match for concrete type i32. You've got a debug binary! Debug binaries support fewer types. Consult https://docs.opendp.org/en/stable/contributing/development-environment.html#build-opendp")
    # Possible resolution:
    #   Should just be the same as any mean without a size.
    context = dp.Context.compositor(
        data=[1, 2, 3, 4, 5],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    mean_query = context.query().clamp((1, 10)).mean()
    mean_query.laplace()


def test_scalar_instead_of_vector():
    with pytest.raises(TypeError, match='To fix, wrap domain kwarg with dp.vector_domain()'):
        dp.Context.compositor(
            data=[1, 2, 3, 4, 5],
            privacy_unit=dp.unit_of(contributions=1),
            privacy_loss=dp.loss_of(epsilon=1.0),
            split_evenly_over=1,
            domain=dp.domain_of(int),
        )

def test_query_dir():
    context = dp.Context.compositor(
        data=[1, 2, 3, 4, 5],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=1,
    )
    query_dir = dir(context.query())
    assert 'count' in query_dir
    assert 'laplace' in query_dir


def test_priv_map_fails():
    pl = pytest.importorskip('polars')
    # partitions? NB_REGIONS = 3
    NB_ROWS = 6

    df = pl.DataFrame(
        {
            "region": [1, 1, 2, 2, 3, 3],
            "income": [1000.0, 1000.0, 2000.0, 2000.0, 3000.0, 3000.0]
        },
        schema={"region": pl.Int64, "income": pl.Float64}
    )
    lf = df.lazy()

    # Minimal domain
    lf_domain = dp.lazyframe_domain([
        dp.series_domain("region", dp.atom_domain(T=dp.i64)),
        dp.series_domain("income", dp.atom_domain(T=dp.f64))
    ])
    lf_domain = dp.with_margin(lf_domain, by=[], public_info="lengths", max_partition_length=NB_ROWS)

    # Group-by query: average income per region, add noïse with scale of 1.0
    income_lower_bound, income_upper_bound = 1_000, 100_000
    plan = lf.group_by("region").agg([
        pl.col("income").dp.mean(bounds=(income_lower_bound, income_upper_bound), scale=1.0)
    ]).sort("income")

    # Bad domain: add a margin that does not include max_num_partitions
    bad_domain = dp.with_margin(lf_domain, by=["region"], public_info="lengths", max_partition_length=NB_ROWS)

    # Construct and collect a measurement with bad_domain works
    # TODO: Should it fail here?
    bad_meas = dp.m.make_private_lazyframe(bad_domain, dp.symmetric_distance(), dp.max_divergence(T=float), plan)
    _ = bad_meas(lf).collect()

    # Privacy map fails: This is expected?
    with pytest.raises(dp.OpenDPException, match='max_num_partitions must be known when the metric is not sensitive to ordering'):
        bad_meas.map(1)