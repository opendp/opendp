from typing import Any, Optional
from opendp._internal import _new_pure_function
from opendp._lib import import_optional_dependency
from opendp.combinators import make_composition
from opendp.extras.polars import dp_len
from opendp.measurements import make_private_lazyframe
from opendp.mod import LazyFrameDomain, Measure, Metric, LazyFrameDomain
from opendp.domains import lazyframe_domain_seed
from opendp.transformations import make_stable_lazyframe


def make_first_order_marginals(
    input_domain: LazyFrameDomain,
    input_metric: Metric,
    output_measure: Measure,
    plan,
    scale: float,
    threshold: int,
):
    m_marginals = [
        make_private_lazyframe(
            input_domain,
            input_metric,
            output_measure,
            plan.group_by(name).agg(dp_len()),
            global_scale=scale,
            threshold=threshold,
        )
        >> _new_pure_function(lambda of: of.collect(), TO="ExtrinsicObject")
        for name in plan.collect_schema()
    ]

    return make_composition(m_marginals)


def make_preprocess_int_array(
    input_domain: LazyFrameDomain,
    input_metric: Metric,
    categories: dict[str, list[Any]],
    plan,
):
    pl = import_optional_dependency("polars")

    return make_stable_lazyframe(
        input_domain,
        input_metric,
        plan.with_columns(
            pl.col(name).replace_strict(
                cats, list(range(len(cats))), default=list(range(len(cats)))
            )
            for name, cats in categories.items()
        ),
    )


def make_synthetic_data(
    input_domain: LazyFrameDomain,
    input_metric: Metric,
    output_measure: Measure,
    categories: dict[str, list[Any]],
    scale,
    threshold,
    plan: Optional[Any] = None,
):
    if plan is None:
        plan = lazyframe_domain_seed(input_domain)

    synth_schema = plan.collect_schema()

    m_marginals = make_first_order_marginals(
        input_domain,
        input_metric,
        output_measure,
        scale,
        threshold,
        plan.with_columns(c for c in input_domain.columns if c not in categories),
    )

    def function(plan):
        marginals = m_marginals(plan)

        categories = [
            marginal[name] for name in zip(marginals, synth_schema) if name in categories
        ]

        make_preprocess_int_array(
            input_domain,
            input_metric,
            categories,
            plan
        )