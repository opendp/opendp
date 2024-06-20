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


@pytest.mark.xfail(raises=TypeError)
def test_int_data_laplace_param():
    # Currently fails with:
    #   TypeError: inferred type is i32, expected f64. See https://github.com/opendp/opendp/discussions/298
    # Possible resolution:
    #   Explicit parameter on laplace works with int data, or the error message should suggest the fix.
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
