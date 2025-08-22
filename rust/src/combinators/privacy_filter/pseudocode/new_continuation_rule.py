# type: ignore
def new_continuation_rule(d_in: MI_Distance, d_out: MO_Distance) -> Wrapper:

    def wrapper(queryable: Queryable) -> Queryable:
        state = queryable
        del queryable  # (moved by rust ownership)

        def transition(query: Query[Any]) -> Answer[Any]:  # `\label{transition}`

            if state is None:
                raise "filter is exhausted"
            else:
                queryable = state

            answer = queryable.eval_query(query)

            match queryable.eval_poly(OdometerQuery.PrivacyLoss(d_in.clone())):  # `\label{external}`
                case OdometerAnswer.PrivacyLoss(pending_d_out):
                    pass
                case _:
                    raise "expected privacy loss"

            if pending_d_out.total_gt(d_out):  # `\label{check}`
                state.take()  # `\label{free}`
                raise "filter is now exhausted"

            return answer   # `\label{return}`
            
        return Queryable.new_raw(transition)

    return Wrapper.new(wrapper)  # `\label{wrapper}`
