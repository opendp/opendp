# type: ignore
def group_by_output_domain(domain, original_keys, keys, aggs):
    schema = domain.schema()
    for (
        agg
    ) in aggs:  # |\label{line:infallible-aggs}\label{line:permutation-invariance}|
        check_permutation_invariance(agg, schema)

    output_schema = domain.simulate_schema(  # |\label{line:simulate-schema}|
        lambda lf: lf.group_by(keys).agg(aggs)
    )
    series_domains = [SeriesDomain.new_from_field(f) for f in output_schema]

    def output_margin(m):
        m = m.clone()
        m.by = materialize_grouping_keys(m.by)  # |\label{line:materialized-margin}|
        m.invariant = None
        return m

    margins = [
        output_margin(m) for m in domain.margins if m.by.is_subset(original_keys)
    ]
    return FrameDomain.new_with_margins(series_domains, margins)


# Called only for subsets of validated grouping expressions.
def materialize_grouping_keys(by: set[Expr]) -> set[Expr]:
    return {col(key.meta().output_name()) for key in by}
