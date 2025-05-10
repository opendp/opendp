# type: ignore
def new_continuation_rule(d_out: U) -> Wrapper:

    def wrapper(queryable: Queryable) -> Queryable:

        def transition(query: Query[Any]) -> Answer[Any]:  # `\label{transition}`

            if isinstance(query, Query.External): # `\label{external}`
                pending_loss: PendingLoss[U] = queryable.eval_internal(query)
                if isinstance(pending_loss, PendingLoss.New):
                    pending_d_out = pending_loss[0]
                    if pending_d_out.total_gt(d_out): # `\label{check}`
                        raise f"insufficient privacy budget: {pending_d_out} > {d_out}"

            return queryable.eval_query(query) # `\label{eval}`

        return Queryable.new_raw(transition)

    return Wrapper.new(wrapper)  # `\label{wrapper}`
