from opendp.measures import max_divergence
from opendp.mod import Measurement, Transformation
from opendp.core import new_user_queryable
from opendp.measurements import make_user_measurement
from opendp.extrinsics._utilities import to_then


def make_stateful_sequential_composition(
    input_domain, input_metric, privacy_measure, d_in, d_mids
):
    if privacy_measure != max_divergence(T=privacy_measure.distance_type):
        raise ValueError("make_stateful_sequential_composition only supports pure-DP")

    if d_in < 0:
        raise ValueError("d_in must not be negative")
    if any(d_i < 0 for d_i in d_mids):
        raise ValueError("d_mids must not be negative")
    
    def function(arg):
        def transition(query, _is_internal):
            nonlocal input_domain, input_metric, arg, d_in, d_mids
            d_mids = list(reversed(d_mids))

            if query.input_space != (input_domain, input_metric):
                raise ValueError(f"expected an input space of {(input_domain, input_metric)}, got {query.input_space}")

            if isinstance(query, Measurement):
                if query.output_measure != privacy_measure:
                    raise ValueError(f"expected a privacy measure of {privacy_measure}, got {query.output_measure}")
                if not query.check(d_in, d_mids[-1]):
                    raise ValueError(f"query consumes {query.map(d_in)}, but is only allowed {d_mids[-1]}")

                answer = query(arg)
                d_mids.pop()
                return answer

            if isinstance(query, Transformation):
                arg, d_in = query.invoke(arg), query.map(d_in)
                input_domain = query.output_domain
                input_metric = query.input_metric
                return
            
            raise ValueError("query must be a Measurement or Transformation")

        return new_user_queryable(transition, Q="ExtrinsicObject", A="ExtrinsicObject")

    d_out = sum(d_mids)

    def privacy_map(d_in_p):
        return d_in_p // d_in * d_out

    return make_user_measurement(
        input_domain,
        input_metric,
        privacy_measure,
        function,
        privacy_map,
        TO="ExtrinsicObject",
    )


then_stateful_sequential_composition = to_then(make_stateful_sequential_composition)
