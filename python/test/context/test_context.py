import pytest
import logging
import opendp.prelude as dp
from opendp._internal import _make_measurement


def test_unit_of():
    assert dp.unit_of(contributions=3) == (dp.symmetric_distance(), 3)
    assert dp.unit_of(contributions=3, ordered=True) == (dp.insert_delete_distance(), 3)

    assert dp.unit_of(changes=3) == (dp.change_one_distance(), 3)
    assert dp.unit_of(changes=3, ordered=True) == (dp.hamming_distance(), 3)

    # For the rest, "ordered=True" is not allowed:

    assert dp.unit_of(absolute=3) == (dp.absolute_distance(dp.i32), 3)
    with pytest.raises(ValueError):
        dp.unit_of(absolute=3, ordered=True)

    assert dp.unit_of(l1=2.0) == (dp.l1_distance(T=float), 2.0)
    with pytest.raises(ValueError):
        dp.unit_of(l1=2.0, ordered=True)

    assert dp.unit_of(l2=2.0) == (dp.l2_distance(T=float), 2.0)
    with pytest.raises(ValueError):
        dp.unit_of(l2=2.0, ordered=True)


def test_privacy_loss_of():
    assert dp.loss_of(epsilon=3) == (dp.max_divergence(), 3.0)
    assert dp.loss_of(rho=2.0) == (dp.zero_concentrated_divergence(), 2.0)
    assert dp.loss_of(epsilon=2.0, delta=1e-6) == (
        dp.approximate(dp.max_divergence()),
        (2.0, 1e-6),
    )
    assert dp.loss_of(rho=0.5, delta=1e-7) == (
        dp.approximate(dp.zero_concentrated_divergence()),
        (0.5, 1e-7),
    )


def test_loss_of_logging(caplog):
    with caplog.at_level(logging.INFO):
        dp.loss_of(epsilon=100.)
        assert caplog.record_tuples == [
            ('opendp.context', logging.WARN, 'epsilon should be less than or equal to 5, and is typically less than or equal to 1')
        ]
        caplog.clear()

        dp.loss_of(epsilon=2., delta=1e-5)
        assert caplog.record_tuples == [
            ('opendp.context', logging.INFO, 'epsilon is typically less than or equal to 1'),
            ('opendp.context', logging.WARN, 'delta should be less than or equal to 1e-06')
        ]
        caplog.clear()

        dp.loss_of(rho=0.3)
        assert caplog.record_tuples == [
            ('opendp.context', logging.INFO, 'rho is typically less than or equal to 0.25')
        ]


def test_option_domain():
    domain = dp.domain_of('Option<int>')
    assert str(domain) == 'OptionDomain(AtomDomain(T=i32))'


def test_context_repr():
    assert repr(
        dp.Context.compositor(
            data=[1, 2, 3],
            privacy_unit=dp.unit_of(contributions=3),
            privacy_loss=dp.loss_of(epsilon=3.0),
            split_evenly_over=1,
            domain=dp.domain_of(list[int]),
        )
    ) == '''Context(
    accountant = Measurement(
        input_domain   = VectorDomain(AtomDomain(T=i32)),
        input_metric   = SymmetricDistance(),
        output_measure = MaxDivergence),
    d_in       = 3,
    d_mids     = [3.0])'''


def test_context_init_split_by_weights():
    dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=3),
        privacy_loss=dp.loss_of(epsilon=3.0),
        split_by_weights=[1, 1, 1],
        domain=dp.domain_of(list[int]),
    )


def test_context_init_split_evenly_over():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=3),
        privacy_loss=dp.loss_of(epsilon=3),
        split_evenly_over=3,
        domain=dp.domain_of(list[int]),
    )

    dp_sum = context.query().clamp((1, 10)).sum().laplace(100.0)  # type: ignore
    print("dp_sum.release()", dp_sum.release())

    # this time the scale parameter is omitted, but it is resolved from the context
    print("context.query().clamp((1, 10)).sum().laplace().release()", context.query().clamp((1, 10)).sum().laplace().release())  # type: ignore
    # where we're headed:
    # print("context.query().dp_sum((1, 10)).release()", context.query().dp_sum((1, 10)).release())



def test_context_zCDP():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0, delta=1e-6),
        split_evenly_over=2,
    )

    dp_sum = context.query().clamp((1, 10)).sum().gaussian(100.0)  # type: ignore
    print("gaussian(100.0)", dp_sum.release())

    dp_sum = context.query().clamp((1, 10)).sum().gaussian()  # type: ignore
    print("gaussian()", dp_sum.release())


def test_middle_param():
    context = dp.Context.compositor(
        data=[1.0, 2.0, 3.0],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0),
        split_evenly_over=1,
    )

    dp_sum = context.query().resize(constant=2.0).clamp((1.0, 10.0)).mean().laplace(1.0)
    # scale = (U - L) / n / ε
    # 1     = (10- 1) / n / 3
    # n     = 9 / 3
    # Due to float rounding, n = 3 results in slightly higher sensitivity,
    # so the lib picks n=4, which is large enough for the sensitivity to be small enough
    # for the query to satisfy ε=3
    dp_sum.param() == 4

    # a sample from Laplace(loc=6 / n, scale=1)
    assert isinstance(dp_sum.release(), float)


def test_query_repr():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.),
        split_evenly_over=1
    )
    assert repr(context.query()) == '''Query(
    chain          = (VectorDomain(AtomDomain(T=i32)), SymmetricDistance()),
    output_measure = MaxDivergence,
    d_in           = 1,
    d_out          = 1.0,
    context        = Context(
        accountant = Measurement(
            input_domain   = VectorDomain(AtomDomain(T=i32)),
            input_metric   = SymmetricDistance(),
            output_measure = MaxDivergence),
        d_in       = 1,
        d_mids     = [1.0]))'''


def test_subcontext_changes_metric():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=2,
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )
    subcontext = context.query().clamp((0, 1)).sum().compositor(split_evenly_over=1).release()

    # This still corresponds to the top-level context:
    assert subcontext.accountant.input_domain == dp.vector_domain(dp.atom_domain(T=dp.i32))

    # TODO: Is there a good way to make this assertion through the public API?
    assert subcontext.query()._chain == (
        dp.atom_domain(T=dp.i32),
        dp.absolute_distance(dp.i32)
    )


def test_measure_cast():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0),
        split_evenly_over=2,
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )
    context.query().compositor(split_evenly_over=1) # TODO: Exercise different output_measure params


def test_split_by_weights_ints():
    dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0, delta=1e-6),
        split_by_weights=[1, 2],
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )


def test_split_by_weights_floats():
    dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0, delta=1e-6),
        split_by_weights=[1.0, 2.0],
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )


def test_sc_query():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=3.0, delta=1e-6),
        split_evenly_over=2,
        # TODO: Using split_by_weights instead fails:
        # split_by_weights=[1, 2],
        domain=dp.vector_domain(dp.atom_domain(T=int)),
    )

    # build a child sequential compositor, and then use it to release a laplace sum
    sub_context_1 = context.query().compositor(split_evenly_over=3).release() # type: ignore[attr-defined]
    dp_sum_1 = sub_context_1.query().clamp((1, 10)).sum().laplace()
    print("laplace dp_sum", dp_sum_1.release())

    # build a child sequential compositor in zCDP, and then use it to release some gaussian queries
    sub_context_2 = context.query().compositor(  # type: ignore[attr-defined]
        split_evenly_over=2,
        output_measure=dp.zero_concentrated_divergence()
    ).release()
    dp_sum_2 = sub_context_2.query().clamp((1, 10)).sum().gaussian()
    # with partials, fusing, and measure convention, would shorten to
    # dp_sum = sub_context_2.query().dp_sum((1, 10))
    print("gaussian dp_sum", dp_sum_2.release())

    dp_mean = (
        sub_context_2.query()
        .cast_default(float)
        .clamp((1.0, 10.0))
        .resize(3, constant=5.0)
        .mean()
        .gaussian()
    )
    # with partials, fusing, and measure convention, would shorten to
    # dp_mean = sub_context.query().cast(float).dp_mean((1., 10.))
    print("gaussian dp_mean", dp_mean.release())


def test_rho_to_eps():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(rho=3.0),
        split_evenly_over=1,
    )

    dp_sum = context.query().clamp((1, 10)).sum().laplace()  # type: ignore

    print("dp_sum.release()", dp_sum.release())


def test_approx_to_approx_zCDP():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(contributions=1),
        privacy_loss=dp.loss_of(epsilon=1.0, delta=1e-6),
        split_evenly_over=1,
    )

    azcdp_measure = dp.approximate(dp.zero_concentrated_divergence())
    context_azcdp = context.query().compositor(3, output_measure=azcdp_measure, alpha=0.3).release()

    dom, met = context_azcdp.accountant.input_space
    # the important test is that the following is a valid query for an approx-zCDP compositor
    release = context_azcdp(_make_measurement(
        dom, met,
        azcdp_measure,
        lambda x: x,
        # remaining rho, and catastrophic delta
        lambda _: (.006, 1e-6 * .3 / 3)
    ))

    assert release == [1, 2, 3]


def test_agg_input():
    context = dp.Context.compositor(
        data=0,
        privacy_unit=dp.unit_of(absolute=1),
        privacy_loss=dp.loss_of(rho=0.5, delta=0.0),
        split_evenly_over=1,
    )

    assert isinstance(context.query().gaussian().release(), int)


def test_rshift_multi_partial():
    context = dp.Context.compositor(
        data=[1, 2, 3],
        privacy_unit=dp.unit_of(l1=1),
        privacy_loss=dp.loss_of(epsilon=0.5),
        split_evenly_over=1,
    )
    with pytest.raises(ValueError, match="laplace is missing 1 parameter"):
        context.query().b_ary_tree(leaf_count=3).laplace()

def test_transformation_release_error():
    privacy_unit = dp.unit_of(contributions=2)
    privacy_loss = dp.loss_of(epsilon=1.0)
    context = dp.Context.compositor(
        data=[1., 2., 3.],
        privacy_unit=privacy_unit,
        privacy_loss=privacy_loss,
        domain=dp.vector_domain(dp.atom_domain(T=float), size=3),
        split_evenly_over=1
    )
    clamped = context.query().clamp((1., 10.))
    with pytest.raises(ValueError, match=r"Query is not yet a measurement"):
        clamped.release()
