# type: ignore
def truncate_domain(
    domain: DslPlanDomain,
    truncation: Truncation,
) -> DslPlanDomain:
    match truncation:
        case Truncation.Filter(_):
            for m in domain.margins:
                # After filtering you no longer know partition lengths or keys.
                m.invariant = None  # |\label{line:invariant-filter}|
            # Upper bounds on the number of rows and groups remain valid.
            return domain
        case Truncation.GroupBy(keys, aggs):
            for agg in aggs:
                # True means resize is allowed
                check_infallible(agg, True)  # |\label{line:infallible}|

            def with_truncation(lf):
                return lf.group_by(keys).agg(aggs)

            def without_invariant(m):
                m.invariant = None
                return m

            # derive new output domain
            return FrameDomain.new_with_margins(
                [  # |\label{line:simulate-schema}|
                    Seriesdomain.new_from_field(f)
                    for f in domain.simulate_schema(with_truncation)
                ],
                margins=[
                    # discard invariants as multiverses are mixed
                    without_invariant(m.clone())
                    for m in domain.margins
                    # only keep margins that are a subset of the truncation keys
                    if m.by.is_subset(HashSet.from_iter(keys))
                ],
            )
