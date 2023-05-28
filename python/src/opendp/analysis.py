from typing import Any, Callable, Optional, Tuple, Union, get_type_hints
import opendp.prelude as dp
import importlib
from inspect import signature
from functools import partial

constructors = {}
for module_name in ["transformations", "measurements"]:
    module = importlib.import_module(f"opendp.{module_name}")
    for name in module.__all__:
        if not name.startswith("make_"):
            continue
        partial_name = "partial_" + name[5:]
        if partial_name in module.__all__:
            constructors[name[5:]] = getattr(module, partial_name), True
        else:
            constructors[name[5:]] = getattr(module, name), False


def privacy_of(*, epsilon=None, delta=None, rho=None, U=None):
    """Standardize privacy loss parameters to (measure, distance)

    >>> from opendp.analysis import privacy_loss_of
    >>> measure, distance = privacy_loss_of(epsilon=1.0)
    >>> measure, distance = privacy_loss_of(epsilon=1.0, delta=1e-9)
    >>> measure, distance = privacy_loss_of(rho=1.0)
    """
    if epsilon is None and rho is None:
        raise ValueError("Either epsilon or rho must be specified.")

    if rho:
        U = dp.RuntimeType.parse_or_infer(U, rho)
        return dp.zero_concentrated_divergence(T=U), rho
    if delta is None:
        U = dp.RuntimeType.parse_or_infer(U, epsilon)
        return dp.max_divergence(T=U), epsilon
    else:
        U = dp.RuntimeType.parse_or_infer(U, epsilon)
        return dp.fixed_smoothed_max_divergence(T=U), (epsilon, delta)


def distance_of(
    *,
    contributions=None,
    changes=None,
    absolute=None,
    l1=None,
    l2=None,
    ordered=False,
    U=None,
):
    def _is_distance(p, v):
        return p not in ["ordered", "U", "_is_distance"] and v is not None

    if sum(1 for p, v in locals().items() if _is_distance(p, v)) != 1:
        raise ValueError("Must specify exactly one distance.")

    if contributions is not None:
        metric = dp.insert_delete_distance() if ordered else dp.symmetric_distance()
        return metric, contributions
    if changes is not None:
        metric = dp.hamming_distance() if ordered else dp.change_one_distance()
        return metric, changes
    if absolute is not None:
        metric = dp.absolute_distance(T=dp.RuntimeType.parse_or_infer(U, absolute))
        return metric, absolute
    if l1 is not None:
        metric = dp.l1_distance(T=dp.RuntimeType.parse_or_infer(U, l1))
        return metric, l1
    if l2 is not None:
        metric = dp.l2_distance(T=dp.RuntimeType.parse_or_infer(U, l2))
        return metric, l2


class Query(object):
    _chain: Union[Tuple[dp.Domain, dp.Metric], dp.Transformation, "PartialChain"]
    _output_measure: dp.Measure
    _analysis: Optional["Analysis"]
    _eager: bool

    def __init__(
        self, input_space, output_measure, d_in, d_out=None, analysis=None, eager=True
    ) -> None:
        self._chain = input_space
        self._output_measure = output_measure
        self._d_in = d_in
        self._d_out = d_out
        self._analysis = analysis
        self._eager = eager

    def __getattr__(self, name: str) -> Callable[[Any], "Query"]:
        if name not in constructors:
            raise AttributeError(f"Unrecognized constructor {name}")
        constructor, is_partial = constructors[name]
        is_measurement = get_type_hints(constructor)["return"] == dp.Measurement

        def make(*args, **kwargs):
            nonlocal constructor, is_partial
            # determine how many parameters are missing
            param_diff = len(args)
            for param in signature(constructor).parameters.values():
                if param.name in kwargs:
                    continue
                if param.default is not param.empty:
                    break
                param_diff -= 1

            if param_diff == -1 and not isinstance(self._chain, PartialChain):
                constructor = PartialChain.wrap(constructor)
            elif param_diff < 0:
                raise ValueError(f"{name} is missing {-param_diff} parameter(s).")
            elif param_diff > 0:
                raise ValueError(f"{name} has {param_diff} parameter(s) too many.")

            new_chain = constructor(*args, **kwargs)
            if is_partial or not isinstance(self._chain, tuple):
                new_chain = self._chain >> new_chain

            new_query = Query(
                input_space=new_chain,
                output_measure=self._output_measure,
                d_in=self._d_in,
                d_out=self._d_out,
                analysis=self._analysis,
                eager=self._eager,
            )
            if self._eager and is_measurement:
                return new_query.release()

            return new_query

        return make

    def __dir__(self):
        return super().__dir__() + list(constructors.keys())

    def resolve(self):
        """Resolve the query into a transformation or measurement."""
        if isinstance(self._chain, PartialChain):
            chain = self._chain.fix(self._d_in, self._d_out)
            if chain.output_measure != self._output_measure:
                raise ValueError("Output measure does not match.")
            return chain
        if isinstance(self._chain, dp.Measurement):
            return self._chain
        raise ValueError("Query is not yet a measurement.")

    def release(self) -> Any:
        return self._analysis(self.resolve())

    def param(self):
        """returns the discovered parameter, if there is one"""
        return getattr(self.resolve(), "param", None)

    def zCDP_to_approxDP(self, map_query: Callable[["Query"], "Query"]) -> "Query":
        new_measure = dp.zero_concentrated_divergence(
            T=self._output_measure.type.args[0]
        )

        def caster(measurement):
            return dp.c.make_fix_delta(
                dp.c.make_zCDP_to_approxDP(measurement), delta=self._d_out[1]
            )

        return self._cast_measure(new_measure, caster, map_query)

    def pureDP_to_fixed_approxDP(
        self, map_query: Callable[["Query"], "Query"]
    ) -> "Query":
        new_measure = dp.fixed_smoothed_max_divergence(
            T=self._output_measure.type.args[0]
        )

        def caster(measurement):
            return dp.c.make_pureDP_to_fixed_approxDP(measurement)

        return self._cast_measure(new_measure, caster, map_query)

    def pureDP_to_zCDP(self, map_query: Callable[["Query"], "Query"]) -> "Query":
        new_measure = dp.zero_concentrated_divergence(
            T=self._output_measure.type.args[0]
        )

        def caster(measurement):
            return dp.c.make_pureDP_to_zCDP(measurement)

        return self._cast_measure(new_measure, caster, map_query)

    def sequential_composition(self, weights) -> "Analysis":
        # TODO: when partial chaining merged, only need to rshift into part_sequential_composition
        if isinstance(self._chain, tuple):
            input_domain, input_metric = self._chain
            d_sc_in = self._d_in
        elif isinstance(self._chain, dp.Transformation):
            input_domain = self._chain.output_domain
            input_metric = self._chain.output_metric
            d_sc_in = self._chain.map(self._d_in)
        else:
            raise ValueError(
                "Sequential composition requires a metric space or transformation."
            )

        if not self._eager:
            raise ValueError("Sequential composition requires eager=True.")

        privacy_unit = input_metric, d_sc_in
        privacy_loss = self._output_measure, self._d_out

        accountant, d_mids = _sequential_composition_accountant(
            input_domain, privacy_unit, privacy_loss, weights
        )
        if isinstance(self._chain, dp.Transformation):
            accountant = self._chain >> accountant
        queryable = self._analysis(accountant)

        return Analysis(accountant=accountant, queryable=queryable, d_in=d_sc_in, d_mids=d_mids)

    def _cast_measure(self, measure, caster, map_query):
        new_query = Query(
            input_space=self._chain,
            output_measure=measure,
            # these can be None because eager=False
            d_in=None,
            d_out=None,
            eager=False,
        )

        inner_chain = map_query(new_query)._chain
        if isinstance(inner_chain, PartialChain):
            casted_chain = PartialChain(lambda x: caster(inner_chain(x)))
        else:
            casted_chain = caster(inner_chain)

        new_query = Query(
            input_space=casted_chain,
            output_measure=self._output_measure,
            d_in=self._d_in,
            d_out=self._d_out,
            analysis=self._analysis,
            eager=self._eager,
        )
        if self._eager:
            return new_query.release()

        return new_query


class PartialChain(object):
    partial: Callable[[Any], dp.Measurement]

    def __init__(self, f, *args, **kwargs):
        self.partial = partial(f, *args, **kwargs)

    def __call__(self, v):
        return self.partial(v)

    def fix(self, d_in, d_out, T=None):
        param = dp.binary_search(lambda x: self.partial(x).check(d_in, d_out), T=T)
        chain = self.partial(param)
        chain.param = param
        return chain

    def __rshift__(self, other):
        if isinstance(other, (dp.Transformation, dp.Measurement)):
            return PartialChain(lambda x: self.partial(x) >> other)

        raise ValueError("other must be a Transformation or Measurement")

    @classmethod
    def wrap(cls, f):
        def inner(*args, **kwargs):
            return cls(f, *args, **kwargs)

        return inner


class Analysis(object):
    compositor: dp.Measurement  # union dp.Odometer once merged
    queryable: dp.Queryable

    def __init__(self, accountant, queryable, d_in, d_mids=None, d_out=None):
        self.accountant = accountant
        self.queryable = queryable
        self.d_in = d_in
        self.d_mids = d_mids
        self.d_out = d_out

    @staticmethod
    def sequential_composition(data, privacy_unit, privacy_loss, weights, domain=None):
        if domain is None:
            # from https://github.com/opendp/opendp/pull/749
            domain = domain_of(data, infer=True)

        accountant, d_mids = _sequential_composition_accountant(
            domain, privacy_unit, privacy_loss, weights
        )

        return Analysis(
            accountant=accountant,
            queryable=accountant(data),
            d_in=privacy_unit[1],
            d_mids=d_mids,
        )

    def __call__(self, query):
        if isinstance(query, Query):
            query = query.resolve()
        answer = self.queryable(query)
        if self.d_mids is not None:
            self.d_mids.pop(0)
        return answer

    def query(self, eager=True, **kwargs) -> Query:
        if self.d_mids is not None:
            if kwargs:
                raise ValueError(f"Expected no privacy arguments, but got {kwargs}.")
            if not self.d_mids:
                raise ValueError("Privacy allowance has been exhausted.")
            d_query = self.d_mids[0]
        else:
            measure, d_query = privacy_of(**kwargs)
            if measure != self.output_measure:
                raise ValueError(
                    f"Expected output measure {self.output_measure} but got {measure}"
                )

        return Query(
            input_space=(self.accountant.input_domain, self.accountant.input_metric),
            output_measure=self.accountant.output_measure,
            d_in=self.d_in,
            d_out=d_query,
            analysis=self,
            eager=eager,
        )


def _sequential_composition_accountant(domain, privacy_unit, privacy_loss, weights):
    input_metric, d_in = privacy_unit
    output_measure, d_out = privacy_loss

    if isinstance(weights, int):
        weights = [d_out] * weights

    def mul(dist, scale):
        if isinstance(dist, tuple):
            return dist[0] * scale, dist[1] * scale
        else:
            return dist * scale

    def scale_weights(scale, weights):
        return [mul(w, scale) for w in weights]

    def scale_sc(scale):
        return dp.c.make_sequential_composition(
            input_domain=domain,
            input_metric=input_metric,
            output_measure=output_measure,
            d_in=d_in,
            d_mids=scale_weights(scale, weights),
        )

    scale = dp.binary_search_param(scale_sc, d_in=d_in, d_out=d_out, T=float)

    # return the accountant and d_mids
    return scale_sc(scale), scale_weights(scale, weights)
