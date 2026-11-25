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
            schema = domain.schema()
            for key in keys:
                if not has_canonical_representatives(key.to_field(schema).dtype):
                    raise ValueError("noncanonical grouping key")
            return group_by_output_domain(  # |\label{line:group-by-domain}|
                domain, set(keys), keys, aggs
            )
