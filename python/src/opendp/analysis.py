from typing import Any, get_type_hints
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


def privacy_loss_of(*, epsilon=None, delta=None, rho=None, U=None):
    """Standardize privacy loss parameters to (measure, distance)

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
    kwargs = {
        ("contributions", contributions),
        ("changes", changes),
        ("absolute", absolute),
        ("l1", l1),
        ("l2", l2),
    }
    kwargs = (x for x in kwargs if x[1] is not None)

    try:
        descriptor, distance = next(kwargs)
    except StopIteration:
        raise ValueError("No distance was specified.")
    if next(kwargs, None):
        raise ValueError("At most one distance can be specified.")

    if descriptor == "contributions":
        metric = dp.insert_delete_distance if ordered else dp.symmetric_distance
        return metric(), distance
    if descriptor == "changes":
        metric = dp.hamming_distance if ordered else dp.change_one_distance
        return metric(), distance

    U = dp.RuntimeType.parse_or_infer(U, distance)
    return {
        "absolute": dp.absolute_distance,
        "l1": dp.l1_distance,
        "l2": dp.l2_distance,
    }[descriptor](T=U), distance


class Query(object):
    def __init__(
        self, input_space, output_measure, d_in, d_out=None, analysis=None, eager=True
    ) -> None:
        self._chain = input_space
        self._output_measure = output_measure
        self._d_in = d_in
        self._d_out = d_out
        self._analysis = analysis
        self._eager = eager

    def __getattr__(self, name: str) -> Any:
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

            new_query = Query(
                input_space=self._chain >> constructor(*args, **kwargs),
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
        return self._analysis.queryable(self.resolve())

    def param(self):
        """returns the discovered parameter, if there is one"""
        return getattr(self.resolve(), "param", None)


class PartialChain(object):
    def __init__(self, f, *args, **kwargs):
        self.partial = partial(f, *args, **kwargs)

    def __call__(self, v):
        return self.partial(v)

    def fix(self, d_in, d_out):
        param = dp.binary_search(lambda x: self.partial(x).check(d_in, d_out))
        chain = self.partial(param)
        chain.param = param
        return chain

    def __rshift__(self, other):
        if isinstance(other, (dp.Transformation, dp.Measurement)):
            return PartialChain(lambda x: self.partial(x) >> other)

        raise ValueError("other must be a Transformation or Measurement")

    def __rrshift__(self, other):
        if isinstance(other, tuple) and list(map(type, other)) == [
            dp.Domain,
            dp.Metric,
        ]:

            def chain(x):
                operation = self.partial(x)
                if (
                    operation.input_domain != other[0]
                    or operation.input_metric != other[1]
                ):
                    raise TypeError(f"Input space {other} does not conform with {self}")

                return operation

            return PartialChain(chain)
        raise TypeError(f"Cannot chain {type(self)} with {type(other)}")

    @classmethod
    def wrap(cls, f):
        def inner(*args, **kwargs):
            return cls(f, *args, **kwargs)

        return inner


class Analysis(object):
    def __init__(
        self,
        input_domain,
        input_metric,
        output_measure,
        data,
        d_in,
        d_mids=None,
        d_out=None,
    ):
        self.d_in = d_in
        self.d_mids = d_mids
        self.d_out = d_out

        if d_mids:
            self.compositor = dp.c.make_sequential_composition(
                input_domain, input_metric, output_measure, d_in, d_mids
            )
            if d_out and not self.compositor.check(d_in, d_out):
                raise ValueError(f"Compositor is not (d_in={d_in}, d_out={d_out})-DP")
        else:
            self.compositor = dp.c.make_sequential_odometer(
                input_domain, input_metric, output_measure
            )
            if d_out:
                self.compositor = dp.c.make_filter(self.compositor, d_in, d_out)
        self.queryable = self.compositor(data)

    def query(self, eager=True, **kwargs) -> Query:
        if self.d_mids is not None:
            d_query = self.d_mids.pop(0)
            if kwargs:
                raise ValueError(f"Expected no privacy arguments, but got {kwargs}.")
        else:
            measure, d_query = privacy_loss_of(**kwargs)
            if measure != self.output_measure:
                raise ValueError(
                    f"Expected output measure {self.output_measure} but got {measure}"
                )

        return Query(
            input_space=(self.compositor.input_domain, self.compositor.input_metric),
            output_measure=self.compositor.output_measure,
            d_in=self.d_in,
            d_out=d_query,
            analysis=self,
            eager=eager,
        )
