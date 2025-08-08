# type: ignore
def new_continuation_rule(d_in: MI_Distance, d_out: MO_Distance) -> Wrapper:

    def wrapper(queryable: Queryable) -> Queryable:

        def transition(query: Query[Any]) -> Answer[Any]:  # `\label{transition}`

            if isinstance(query, Query.External): # `\label{external}`
                pending_map: PendingLoss[PrivacyMap[MI, MO]] = queryable.eval_internal(query)
                if isinstance(pending_map, PendingLoss.New):

                    pending_d_out = pending_map.eval(d_in)
                    if pending_d_out.total_gt(d_out): # `\label{check}`
                        raise f"insufficient privacy budget: {pending_d_out} > {d_out}"

            return queryable.eval_query(query) # `\label{eval}`

        return Queryable.new_raw(transition)

    return Wrapper.new(wrapper)  # `\label{wrapper}`
