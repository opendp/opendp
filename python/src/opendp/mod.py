'''
The ``mod`` module provides the classes which implement the
`OpenDP Programming Framework <../../user/programming-framework/index.html>`_,
as well as utilities for enabling features and finding parameter values.

The classes here correspond to other top-level modules: For example,
instances of :py:class:`opendp.mod.Domain` are either inputs or outputs for functions in :py:mod:`opendp.domains`.
'''
from __future__ import annotations
import ctypes
from typing import Any, Literal, Type, TypeVar, Union, Tuple, Callable, Optional, overload, TYPE_CHECKING

from opendp._lib import AnyMeasurement, AnyTransformation, AnyDomain, AnyMetric, AnyMeasure, AnyFunction

# https://mypy.readthedocs.io/en/stable/runtime_troubles.html#import-cycles
if TYPE_CHECKING:
    from opendp.typing import RuntimeType # pragma: no cover


class Measurement(ctypes.POINTER(AnyMeasurement)): # type: ignore[misc]
    """A differentially private unit of computation.
    A measurement contains a function and a privacy relation.
    The function releases a differentially-private release.
    The privacy relation maps from an input metric to an output measure.

    See the `Measurement <../../user/programming-framework/core-structures.html#measurement>`_
    section in the Programming Framework docs for more context.

    Functions for creating measurements are in :py:mod:`opendp.measurements`.

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    >>> # create an instance of Measurement using a constructor from the meas module
    >>> base_dl: dp.Measurement = dp.m.make_base_discrete_laplace(
    ...     dp.atom_domain(T=int), dp.absolute_distance(T=int),
    ...     scale=2.)

    >>> # invoke the measurement (invoke and __call__ are equivalent)
    >>> print('explicit: ', base_dl.invoke(100))  # -> 101   # doctest: +ELLIPSIS
    explicit: ...
    >>> print('concise: ', base_dl(100))  # -> 99            # doctest: +ELLIPSIS
    concise: ...
    >>> # check the measurement's relation at
    >>> #     (1, 0.5): (AbsoluteDistance<u32>, MaxDivergence)
    >>> assert base_dl.check(1, 0.5)

    >>> # chain with a transformation from the trans module
    >>> chained = (
    ...     (dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()) >>
    ...     dp.t.then_count() >>
    ...     base_dl
    ... )

    >>> # the resulting measurement has the same features
    >>> print('dp count: ', chained([1, 2, 3]))  # -> 4     # doctest: +ELLIPSIS
    dp count: ...

    >>> # check the chained measurement's relation at
    >>> #     (1, 0.5): (SymmetricDistance, MaxDivergence)
    >>> assert chained.check(1, 0.5)
    """
    _type_ = AnyMeasurement

    def __call__(self, arg):
        from opendp.core import measurement_invoke
        return measurement_invoke(self, arg)

    def invoke(self, arg):
        """Create a differentially-private release with `arg`.

        If `self` is (d_in, d_out)-close, then each invocation of this function is a d_out-DP release. 
        
        :param arg: Input to the measurement.
        :return: differentially-private release
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import measurement_invoke
        return measurement_invoke(self, arg)

    def map(self, d_in):
        """Map an input distance `d_in` to an output distance."""
        from opendp.core import measurement_map
        return measurement_map(self, d_in)

    def check(self, d_in, d_out, *, debug=False) -> bool:
        """Check if the measurement is (`d_in`, `d_out`)-close.
        If true, implies that if the distance between inputs is at most `d_in`, then the privacy usage is at most `d_out`.
        See also :func:`~Transformation.check`, a similar check for transformations.
        
        :param d_in: Distance in terms of the input metric.
        :param d_out: Distance in terms of the output measure.
        :param debug: Enable to raise Exceptions to help identify why the privacy relation failed.
        :return: If True, a release is differentially private at `d_in`, `d_out`.
        :rtype: bool
        """
        from opendp.core import measurement_check

        if debug:
            return measurement_check(self, d_in, d_out)

        try:
            return measurement_check(self, d_in, d_out)
        except OpenDPException as err:
            if err.variant == "RelationDebug":
                return False
            raise

    def __rshift__(self, other: Union["Function", "Transformation"]) -> "Measurement":
        if isinstance(other, Transformation):
            other = other.function

        if not isinstance(other, Function):
            from opendp.core import new_function
            other = new_function(other, TO="ExtrinsicObject")

        if isinstance(other, Function):
            from opendp.combinators import make_chain_pm
            return make_chain_pm(other, self)

        raise ValueError(f"rshift expected a postprocessing transformation, got {other}")

    @property
    def input_domain(self) -> "Domain":
        from opendp.core import measurement_input_domain
        return measurement_input_domain(self)
    
    @property
    def input_metric(self) -> "Metric":
        from opendp.core import measurement_input_metric
        return measurement_input_metric(self)

    @property
    def input_space(self) -> Tuple["Domain", "Metric"]:
        return self.input_domain, self.input_metric
    
    @property
    def output_measure(self) -> "Measure":
        from opendp.core import measurement_output_measure
        return measurement_output_measure(self)
    
    @property
    def function(self) -> "Function":
        from opendp.core import measurement_function
        return measurement_function(self)
    
    @property
    def input_distance_type(self) -> Union["RuntimeType", str]:
        """Retrieve the distance type of the input metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.
        
        :return: distance type
        """
        from opendp.core import measurement_input_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measurement_input_distance_type(self))

    @property
    def output_distance_type(self) -> Union["RuntimeType", str]:
        """Retrieve the distance type of the output measure.
        This is the type that the budget is expressed in.
        
        :return: distance type
        """
        from opendp.core import measurement_output_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measurement_output_distance_type(self))

    @property
    def input_carrier_type(self) -> Union["RuntimeType", str]:
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.
        
        :return: carrier type
        """
        from opendp.core import measurement_input_carrier_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measurement_input_carrier_type(self))

    def _depends_on(self, *args):
        """Extends the memory lifetime of args to the lifetime of self."""
        setattr(self, "_dependencies", args)

    def __del__(self):
        try:
            from opendp.core import _measurement_free
            _measurement_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass
    
    def __str__(self) -> str:
        return f"Measurement(\n    input_domain   = {self.input_domain}, \n    input_metric   = {self.input_metric}, \n    output_measure = {self.output_measure}\n)" # pragma: no cover


class Transformation(ctypes.POINTER(AnyTransformation)): # type: ignore[misc]
    """A non-differentially private unit of computation.
    A transformation contains a function and a stability relation.
    The function maps from an input domain to an output domain.
    The stability relation maps from an input metric to an output metric.

    See the `Transformation <../../user/programming-framework/core-structures.html#transformation>`_
    section in the Programming Framework docs for more context.

    Functions for creating transformations are in :py:mod:`opendp.transformations`.

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    >>> # create an instance of Transformation using a constructor from the trans module
    >>> input_space = (dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
    >>> count: dp.Transformation = input_space >> dp.t.then_count()

    >>> # invoke the transformation (invoke and __call__ are equivalent)
    >>> count.invoke([1, 2, 3])
    3
    >>> count([1, 2, 3])
    3
    >>> # check the transformation's relation at
    >>> #     (1, 1): (SymmetricDistance, AbsoluteDistance<u32>)
    >>> assert count.check(1, 1)

    >>> # chain with more transformations from the trans module
    >>> chained = (
    ...     dp.t.make_split_lines() >>
    ...     dp.t.then_cast_default(TOA=int) >>
    ...     count
    ... )

    >>> # the resulting transformation has the same features
    >>> chained("1\\n2\\n3")
    3
    >>> assert chained.check(1, 1)  # both chained transformations were 1-stable
    """
    _type_ = AnyTransformation

    def invoke(self, arg):
        """Execute a non-differentially-private query with `arg`.

        :param arg: Input to the transformation.
        :return: non-differentially-private answer
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import transformation_invoke # pragma: no cover
        return transformation_invoke(self, arg)  # pragma: no cover

    def __call__(self, arg):
        from opendp.core import transformation_invoke
        return transformation_invoke(self, arg)

    def map(self, d_in):
        """Map an input distance `d_in` to an output distance."""
        from opendp.core import transformation_map
        return transformation_map(self, d_in)

    def check(self, d_in, d_out, *, debug=False):
        """Check if the transformation is (`d_in`, `d_out`)-close.
        If true, implies that if the distance between inputs is at most `d_in`, then the distance between outputs is at most `d_out`.
        See also :func:`~Measurement.check`, a similar check for measurements.

        :param d_in: Distance in terms of the input metric.
        :param d_out: Distance in terms of the output metric.
        :param debug: Enable to raise Exceptions to help identify why the stability relation failed.
        :return: True if the relation passes. False if the relation failed.
        :rtype: bool
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import transformation_check

        if debug:
            return transformation_check(self, d_in, d_out) # pragma: no cover

        try:
            return transformation_check(self, d_in, d_out)
        except OpenDPException as err:
            if err.variant == "RelationDebug":
                return False
            raise

    @overload
    def __rshift__(self, other: "Transformation") -> "Transformation":
        ...

    @overload
    def __rshift__(self, other: "Measurement") -> "Measurement":
        ...

    @overload
    def __rshift__(self, other: "PartialConstructor") -> "PartialConstructor":
        ...

    def __rshift__(self, other: Union["Measurement", "Transformation", "PartialConstructor"]) -> Union["Measurement", "Transformation", "PartialConstructor", "PartialChain"]:  # type: ignore[name-defined] # noqa F821
        if isinstance(other, Measurement):
            from opendp.combinators import make_chain_mt
            return make_chain_mt(other, self)

        if isinstance(other, Transformation):
            from opendp.combinators import make_chain_tt
            return make_chain_tt(other, self)
        
        if isinstance(other, PartialConstructor):
            return self >> other(self.output_domain, self.output_metric) # type: ignore[call-arg]

        from opendp.context import PartialChain
        if isinstance(other, PartialChain):
            return PartialChain(lambda x: self >> other.partial(x))

        raise ValueError(f"rshift expected a measurement or transformation, got {other}")


    @property
    def input_domain(self) -> "Domain":
        from opendp.core import transformation_input_domain
        return transformation_input_domain(self)
    

    @property
    def output_domain(self) -> "Domain":
        from opendp.core import transformation_output_domain
        return transformation_output_domain(self)
    

    @property
    def input_metric(self) -> "Metric":
        from opendp.core import transformation_input_metric
        return transformation_input_metric(self)
    
    @property
    def output_metric(self) -> "Metric":
        from opendp.core import transformation_output_metric
        return transformation_output_metric(self)
    
    @property
    def input_space(self) -> Tuple["Domain", "Metric"]:
        return self.input_domain, self.input_metric # pragma: no cover
    
    @property
    def output_space(self) -> Tuple["Domain", "Metric"]:
        return self.output_domain, self.output_metric
    
    @property
    def function(self) -> "Function":
        from opendp.core import transformation_function # pragma: no cover
        return transformation_function(self) # pragma: no cover

    @property
    def input_distance_type(self) -> Union["RuntimeType", str]:
        """Retrieve the distance type of the input metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.

        :return: distance type
        """
        from opendp.core import transformation_input_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(transformation_input_distance_type(self))

    @property
    def output_distance_type(self) -> Union["RuntimeType", str]:
        """Retrieve the distance type of the output metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.

        :return: distance type
        """
        from opendp.core import transformation_output_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(transformation_output_distance_type(self))
    
    @property
    def input_carrier_type(self) -> Union["RuntimeType", str]:
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.

        :return: carrier type
        """
        from opendp.core import transformation_input_carrier_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(transformation_input_carrier_type(self))

    def _depends_on(self, *args):
        """Extends the memory lifetime of args to the lifetime of self."""
        setattr(self, "_dependencies", args)

    def __del__(self):
        try:
            from opendp.core import _transformation_free
            _transformation_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __str__(self) -> str:
        return f"Transformation(\n    input_domain   = {self.input_domain},\n    output_domain  = {self.output_domain},\n    input_metric   = {self.input_metric},\n    output_metric  = {self.output_metric}\n)"

from typing import cast
Transformation = cast(Type[Transformation], Transformation) # type: ignore[misc]

class Queryable(object):
    def __init__(self, value):
        self.value = value

    def __call__(self, query):
        from opendp.core import queryable_eval
        return queryable_eval(self.value, query)
    
    def eval(self, query):
        from opendp.core import queryable_eval # pragma: no cover
        return queryable_eval(self.value, query) # pragma: no cover

    @property
    def query_type(self):
        from opendp.core import queryable_query_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(queryable_query_type(self.value))

    def __str__(self):
        return f"Queryable(Q={self.query_type})"

    def _depends_on(self, *args):
        """Extends the memory lifetime of args to the lifetime of self."""
        setattr(self, "_dependencies", args)
        

class Function(ctypes.POINTER(AnyFunction)): # type: ignore[misc]
    '''
    See the `Function <../../user/programming-framework/supporting-elements.html#function>`_
    section in the Programming Framework docs for more context.
    '''
    _type_ = AnyFunction

    def __call__(self, arg):
        from opendp.core import function_eval
        return function_eval(self, arg)
    
    def _depends_on(self, *args):
        """Extends the memory lifetime of args to the lifetime of self."""
        setattr(self, "_dependencies", args)
    
    def __del__(self):
        try:
            from opendp.core import _function_free
            _function_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass


class Domain(ctypes.POINTER(AnyDomain)): # type: ignore[misc]
    '''
    See the `Domain <../../user/programming-framework/supporting-elements.html#domain>`_
    section in the Programming Framework docs for more context.

    Functions for creating domains are in :py:mod:`opendp.domains`.
    '''
    _type_ = AnyDomain

    def member(self, val):
        from opendp.domains import member
        return member(self, val)

    @property
    def type(self) -> Union["RuntimeType", str]:
        from opendp.domains import domain_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(domain_type(self))
    
    @property
    def carrier_type(self) -> Union["RuntimeType", str]:
        from opendp.domains import domain_carrier_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(domain_carrier_type(self))
    
    @property
    def descriptor(self) -> Any:
        from opendp.domains import _user_domain_descriptor
        return _user_domain_descriptor(self)

    def __str__(self):
        from opendp.domains import domain_debug
        return domain_debug(self)
    
    def __del__(self):
        try:
            from opendp.domains import _domain_free
            _domain_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __repr__(self) -> str:
        return str(self)
    
    def __eq__(self, other) -> bool:
        # TODO: consider adding ffi equality
        return str(self) == str(other)
    
    def __hash__(self) -> int:
        return hash(str(self))
    
    def _depends_on(self, *args):
        """Extends the memory lifetime of args to the lifetime of self."""
        setattr(self, "_dependencies", args)



class Metric(ctypes.POINTER(AnyMetric)): # type: ignore[misc]
    '''
    See the `Metric <../../user/programming-framework/supporting-elements.html#metric>`_
    section in the Programming Framework docs for more context.

    Functions for creating metrics are in :py:mod:`opendp.metrics`.
    '''
    _type_ = AnyMetric

    @property
    def type(self):
        from opendp.metrics import metric_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(metric_type(self))
    
    @property
    def distance_type(self) -> Union["RuntimeType", str]:
        from opendp.metrics import metric_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(metric_distance_type(self))

    def __str__(self):
        from opendp.metrics import metric_debug
        return metric_debug(self)
    
    def __del__(self):
        try:
            from opendp.metrics import _metric_free
            _metric_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __repr__(self) -> str:
        return str(self)
    
    def __eq__(self, other) -> bool:
        # TODO: consider adding ffi equality
        return str(self) == str(other)
    
    def __hash__(self) -> int:
        return hash(str(self))


class Measure(ctypes.POINTER(AnyMeasure)): # type: ignore[misc]
    '''
    See the `Measure <../../user/programming-framework/supporting-elements.html#measure>`_
    section in the Programming Framework docs for more context.

    Functions for creating measures are in :py:mod:`opendp.measures`.
    '''
    _type_ = AnyMeasure

    @property
    def type(self):
        from opendp.measures import measure_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measure_type(self))
    
    @property
    def distance_type(self) -> Union["RuntimeType", str]:
        from opendp.measures import measure_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measure_distance_type(self))

    def __str__(self):
        from opendp.measures import measure_debug
        return measure_debug(self)
    
    def __del__(self):
        try:
            from opendp.measures import _measure_free
            _measure_free(self)
        except (ImportError, TypeError):
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __eq__(self, other):
        return str(self) == str(other)
    
    def __hash__(self) -> int:
        return hash(str(self))


class SMDCurve(object):
    def __init__(self, curve):
        self.curve = curve

    def epsilon(self, delta):
        from opendp._data import smd_curve_epsilon
        return smd_curve_epsilon(self.curve, delta)


class PartialConstructor(object):
    def __init__(self, constructor):
        self.constructor = constructor
    
    def __call__(self, input_domain: Domain, input_metric: Metric):
        return self.constructor(input_domain, input_metric)
    
    def __rshift__(self, other):
        return PartialConstructor(lambda input_domain, input_metric: self(input_domain, input_metric) >> other) # pragma: no cover

    def __rrshift__(self, other):
        if isinstance(other, tuple) and list(map(type, other)) == [Domain, Metric]:
            return self(other[0], other[1])
        raise TypeError(f"Cannot chain {type(self)} with {type(other)}")


class UnknownTypeException(Exception):
    pass


class OpenDPException(Exception):
    """General exception for errors originating from the underlying OpenDP library.
    The variant attribute corresponds to `one of the following variants <https://github.com/opendp/opendp/blob/53ec58d01762ca5ceee08590d7e7b725bbdafcf6/rust/opendp/src/error.rs#L46-L87>`_ and can be matched on.
    Error variants may change in library updates.

    See `Rust ErrorVariant <https://docs.rs/opendp/latest/opendp/error/enum.ErrorVariant.html>`_ for values variant may take on.
    """
    raw_traceback: Optional[str]

    def __init__(self, variant: str, message: Optional[str] = None, raw_traceback: Optional[str] = None):
        self.variant = variant
        self.message = message
        self.raw_traceback = raw_traceback

    def raw_frames(self): # pragma: no cover
        import re
        return re.split(r"\s*[0-9]+: ", self.raw_traceback or "")
    
    def frames(self): # pragma: no cover
        def format_frame(frame):
            return "\n  ".join(l.strip() for l in frame.split("\n"))
        return [format_frame(f) for f in self.raw_frames() if f.startswith("opendp") or f.startswith("<opendp")]

    def __str__(self) -> str: # pragma: no cover
        response = ''
        if self.raw_traceback and 'rust-stack-trace' in GLOBAL_FEATURES:
            # join and split by newlines because frames may be multi-line
            lines = "\n".join(self.frames()[::-1]).split('\n')
            response += "Continued Rust stack trace:\n" + '\n'.join('    ' + line for line in lines)

        response += '\n  ' + self.variant

        if self.message:
            response += f'("{self.message}")'
            
        return response


GLOBAL_FEATURES = set()


def enable_features(*features: str) -> None:
    GLOBAL_FEATURES.update(set(features))


def disable_features(*features: str) -> None:
    GLOBAL_FEATURES.difference_update(set(features))


def assert_features(*features: str) -> None:
    for feature in features:
        assert feature in GLOBAL_FEATURES, f"Attempted to use function that requires {feature}, but {feature} is not enabled. See https://github.com/opendp/opendp/discussions/304, then call enable_features(\"{feature}\")"


M = TypeVar("M", Transformation, Measurement)

def binary_search_chain(
        make_chain: Callable[[float], M],
        d_in: Any, d_out: Any,
        bounds: Tuple[float, float] | None = None,
        T=None) -> M:
    """Find the highest-utility (`d_in`, `d_out`)-close Transformation or Measurement.
    
    Searches for the numeric parameter to `make_chain` that results in a computation
    that most tightly satisfies `d_out` when datasets differ by at most `d_in`,
    then returns the Transformation or Measurement corresponding to said parameter.

    See `binary_search_param` to retrieve the discovered parameter instead of the complete computation chain.

    :param make_chain: a function that takes a number and returns a Transformation or Measurement
    :param d_in: how far apart input datasets can be
    :param d_out: how far apart output datasets or distributions can be
    :param bounds: a 2-tuple of the lower and upper bounds on the input of `make_chain`
    :param T: type of argument to `make_chain`, one of {float, int}
    :return: a chain parameterized at the nearest passing value to the decision point of the relation
    :rtype: Union[Transformation, Measurement]
    :raises TypeError: if the type is not inferrable (pass T) or the type is invalid
    :raises ValueError: if the predicate function is constant, bounds cannot be inferred, or decision boundary is not within `bounds`.


    :examples:

    Find a base_laplace measurement with the smallest noise scale that is still (d_in, d_out)-close.

    >>> from typing import List
    >>> import opendp.prelude as dp
    >>> dp.enable_features("floating-point", "contrib")
    ...
    >>> # The majority of the chain only needs to be defined once.
    >>> pre = (
    ...     dp.space_of(List[float]) >>
    ...     dp.t.then_clamp(bounds=(0., 1.)) >>
    ...     dp.t.then_resize(size=10, constant=0.) >>
    ...     dp.t.then_mean()
    ... )
    ...
    >>> # Find a value in `bounds` that produces a (`d_in`, `d_out`)-chain nearest the decision boundary.
    >>> # The lambda function returns the complete computation chain when given a single numeric parameter.
    >>> chain = dp.binary_search_chain(
    ...     lambda s: pre >> dp.m.then_base_laplace(scale=s), 
    ...     d_in=1, d_out=1.)
    ...
    >>> # The resulting computation chain is always (`d_in`, `d_out`)-close, but we can still double-check:
    >>> assert chain.check(1, 1.)


    Build a (2 neighboring, 1. epsilon)-close sized bounded sum with discrete_laplace(100.) noise.
    It should have the widest possible admissible clamping bounds (-b, b).

    >>> def make_sum(b):
    ...     space = dp.vector_domain(dp.atom_domain((-b, b)), 10_000), dp.symmetric_distance()
    ...     return space >> dp.t.then_sum() >> dp.m.then_laplace(100.)
    ...
    >>> # `meas` is a Measurement with the widest possible clamping bounds.
    >>> meas = dp.binary_search_chain(make_sum, d_in=2, d_out=1., bounds=(0, 10_000))
    ...
    >>> # If you want the discovered clamping bound, use `binary_search_param` instead.
    """
    return make_chain(binary_search_param(make_chain, d_in, d_out, bounds, T))


def binary_search_param(
        make_chain: Callable[[float], Union[Transformation, Measurement]],
        d_in: Any, d_out: Any,
        bounds: Tuple[float, float] | None = None,
        T=None) -> float:
    """Solve for the ideal constructor argument to `make_chain`.
    
    Optimizes a parameterized chain `make_chain` within float or integer `bounds`,
    subject to the chained relation being (`d_in`, `d_out`)-close.

    :param make_chain: a function that takes a number and returns a Transformation or Measurement
    :param d_in: how far apart input datasets can be
    :param d_out: how far apart output datasets or distributions can be
    :param bounds: a 2-tuple of the lower and upper bounds on the input of `make_chain`
    :param T: type of argument to `make_chain`, one of {float, int}
    :return: the nearest passing value to the decision point of the relation
    :raises TypeError: if the type is not inferrable (pass T) or the type is invalid
    :raises ValueError: if the predicate function is constant, bounds cannot be inferred, or decision boundary is not within `bounds`.

    :example:

    >>> import opendp.prelude as dp
    ...
    >>> # Find a value in `bounds` that produces a (`d_in`, `d_out`)-chain nearest the decision boundary.
    >>> # The first argument is any function that returns your complete computation chain
    >>> #     when passed a single numeric parameter.
    ...
    >>> def make_fixed_laplace(scale):
    ...     # fixes the input domain and metric, but parameterizes the noise scale
    ...     return dp.m.make_base_laplace(dp.atom_domain(T=float), dp.absolute_distance(T=float), scale)
    ...
    >>> scale = dp.binary_search_param(make_fixed_laplace, d_in=0.1, d_out=1.)
    >>> assert scale == 0.1
    >>> # Constructing the same chain with the discovered parameter will always be (0.1, 1.)-close.
    >>> assert make_fixed_laplace(scale).check(0.1, 1.)

    A policy research organization wants to know the smallest sample size necessary to release an "accurate" epsilon=1 DP mean income. 
    Determine the smallest dataset size such that, with 95% confidence, 
    the DP release differs from the clipped dataset's mean by no more than 1000. 
    Assume that neighboring datasets have a symmetric distance at most 2. 
    Also assume a clipping bound of 500,000.

    >>> # we first work out the necessary noise scale to satisfy the above constraints.
    >>> necessary_scale = dp.accuracy_to_laplacian_scale(accuracy=1000., alpha=.05)
    ...
    >>> # we then write a function that make a computation chain with a given data size
    >>> def make_mean(data_size):
    ...    return (
    ...        (dp.vector_domain(dp.atom_domain(bounds=(0., 500_000.)), data_size), dp.symmetric_distance()) >>
    ...        dp.t.then_mean() >> 
    ...        dp.m.then_base_laplace(necessary_scale)
    ...    )
    ...
    >>> # solve for the smallest dataset size that admits a (2 neighboring, 1. epsilon)-close measurement
    >>> dp.binary_search_param(
    ...     make_mean, 
    ...     d_in=2, d_out=1.,
    ...     bounds=(1, 1000000))
    1498
    """

    # one might think running scipy.optimize.brent* would be better, but 
    # 1. benchmarking showed no difference or minor regressions
    # 2. brentq is more complicated

    return binary_search(lambda param: make_chain(param).check(d_in, d_out), bounds, T)

# when return sign is false, only return float
@overload
def binary_search(
        predicate: Callable[[float], bool],
        bounds: Tuple[float, float] | None = ...,
        T: Type[float] | None = ...,
        return_sign: Literal[False] = False) -> float:
    ...


# when setting return sign to true as a keyword argument, return both
@overload
def binary_search(
        predicate: Callable[[float], bool],
        bounds: Tuple[float, float] | None = ...,
        T: Type[float] | None = ...,
        *, # see https://stackoverflow.com/questions/66435480/overload-following-optional-argument
        return_sign: Literal[True]) -> Tuple[float, int]:
    ...

# when setting return sign to true as a positional argument, return both
@overload
def binary_search(
        predicate: Callable[[float], bool],
        bounds: Tuple[float, float] | None,
        T: Type[float] | None,
        return_sign: Literal[True]) -> Tuple[float, int]:
    ...


def binary_search(
        predicate: Callable[[float], bool],
        bounds: Tuple[float, float] | None = None,
        T: Type[float] | None = None,
        return_sign: bool = False) -> float | Tuple[float, int]:
    """Find the closest passing value to the decision boundary of `predicate`.

    If bounds are not passed, conducts an exponential search.
    
    :param predicate: a monotonic unary function from a number to a boolean
    :param bounds: a 2-tuple of the lower and upper bounds to the input of `predicate`
    :param T: type of argument to `predicate`, one of {float, int}
    :param return_sign: if True, also return the direction away from the decision boundary
    :return: the discovered parameter within the bounds
    :raises TypeError: if the type is not inferrable (pass T) or the type is invalid
    :raises ValueError: if the predicate function is constant, bounds cannot be inferred, or decision boundary is not within `bounds`.

    :example:

    >>> from opendp.mod import binary_search
    >>> binary_search(lambda x: x >= 5.)
    5.0
    >>> binary_search(lambda x: x <= 5.)
    5.0

    >>> binary_search(lambda x: x > 5, T=int)
    6
    >>> binary_search(lambda x: x < 5, T=int)
    4

    Find epsilon usage of the gaussian(scale=1.) mechanism applied on a dp mean.
    Assume neighboring datasets differ by up to three additions/removals, and fix delta to 1e-8.

    >>> # build a histogram that emits float counts
    >>> import opendp.prelude as dp
    >>> input_space = dp.vector_domain(dp.atom_domain(bounds=(0., 100.)), 1000), dp.symmetric_distance()
    >>> dp_mean = dp.c.make_fix_delta(dp.c.make_zCDP_to_approxDP(
    ...     input_space >> dp.t.then_mean() >> dp.m.then_gaussian(1.)), 
    ...     1e-8
    ... )
    ...
    >>> dp.binary_search(
    ...     lambda d_out: dp_mean.check(3, (d_out, 1e-8)), 
    ...     bounds = (0., 1.))
    0.5235561269546629

    Find the L2 distance sensitivity of a histogram when neighboring datasets differ by up to 3 additions/removals.

    >>> from opendp.transformations import make_count_by_categories
    >>> histogram = dp.t.make_count_by_categories(
    ...     dp.vector_domain(dp.atom_domain(T=str)), dp.symmetric_distance(),
    ...     categories=["a"], MO=dp.L2Distance[int])
    ...
    >>> dp.binary_search(
    ...     lambda d_out: histogram.check(3, d_out), 
    ...     bounds = (0, 100))
    3
    """
    if bounds is None:
        bounds = exponential_bounds_search(predicate, T) # type: ignore

    if bounds is None:
        raise ValueError("unable to infer bounds")

    if len(set(map(type, bounds))) != 1:
        raise TypeError("bounds must share the same type")
    lower, upper = sorted(bounds)

    maximize = predicate(lower)  # if the lower bound passes, we should maximize
    minimize = predicate(upper)  # if the upper bound passes, we should minimize
    if maximize == minimize:
        raise ValueError("the decision boundary of the predicate is outside the bounds")

    if isinstance(lower, float):
        tolerance = 0.
        half = lambda x: x / 2.
    elif isinstance(lower, int):
        tolerance = 1  # the lower and upper bounds never meet due to int truncation
        half = lambda x: x // 2
    else:
        raise TypeError("bounds must be either float or int")

    mid = lower
    while upper - lower > tolerance:
        new_mid = lower + half(upper - lower)  # avoid overflow

        # avoid an infinite loop from float roundoff
        if new_mid == mid:
            break

        mid = new_mid
        if predicate(mid) == minimize:
            upper = mid
        else:
            lower = mid

    # one bound is always false, the other true. Return the truthy bound
    value = upper if minimize else lower

    # optionally return sign
    if return_sign:
        return value, 1 if minimize else -1 # type: ignore
    
    return value


def exponential_bounds_search(
    predicate: Callable[[Union[float, int]], bool], 
    T: Optional[Union[Type[float], Type[int]]]) -> Optional[Union[Tuple[float, float], Tuple[int, int]]]:
    """Determine bounds for a binary search via an exponential search,
    in large bands of [2^((k - 1)^2), 2^(k^2)] for k in [0, 8).
    Will attempt to recover once if `predicate` throws an exception, 
    by searching bands on the ok side of the exception boundary.
    

    :param predicate: a monotonic unary function from a number to a boolean
    :param T: type of argument to predicate, one of {float, int}
    :return: a tuple of float or int bounds that the decision boundary lies within
    :raises TypeError: if the type is not inferrable (pass T)
    :raises ValueError: if the predicate function is constant
    """

    # try to infer T
    if T is None:
        def check_type(v):
            try:
                predicate(v)
            except TypeError as e:
                return False
            except OpenDPException as e:
                if "No match for concrete type" in (e.message or ""):
                    return False
            return True
        
        if check_type(0.):
            T = float
        elif check_type(0):
            T = int
        else:
            raise TypeError("unable to infer type `T`; pass the type `T` or bounds")

    # core search functionality
    def signed_band_search(center, at_center, sign):
        """identify which band (of eight) the decision boundary lies in, 
        starting from `center` in the direction indicated by `sign`"""

        if T == int:
            # searching bands of [(k - 1) * 2^16, k * 2^16].
            # center + 1 included because zero is prone to error
            bands = [center, center + 1, *(center + sign * 2 ** 16 * k for k in range(1, 9))]

        elif T == float:
            # searching bands of [2^((k - 1)^2), 2^(k^2)].
            # exponent has ten bits (2.^1024 overflows) so k must be in [0, 32).
            # unlikely to need numbers greater than 2**64, and to avoid overflow from shifted centers,
            #    only check k in [0, 8). Set your own bounds if this is not sufficient
            bands = [center, *(center + sign * 2. ** k ** 2 for k in range(1024 // 32 // 4))]
        else:
            raise TypeError(f"unknown type {T}. Must be one of int, float")

        for i in range(1, len(bands)):
            # looking for a change in sign that indicates the decision boundary is within this band
            if at_center != predicate(bands[i]):
                # return the band
                return tuple(sorted(bands[i - 1:i + 1]))
        
        # No band found!
        return None

    center = {int: 0, float: 0.}[T]
    try:
        at_center = predicate(center)
        # search positive bands, then negative bands
        return signed_band_search(center, at_center, 1) or signed_band_search(center, at_center, -1)
    except Exception:
        pass


    # predicate has thrown an exception
    # 1. Treat exceptions as a secondary decision boundary, and find the edge value
    # 2. Return a bound by searching from the exception edge, in the direction away from the exception
    def exception_predicate(v):
        try:
            predicate(v)
            return True
        except Exception:
            return False
    exception_bounds = exponential_bounds_search(exception_predicate, T=T)
    if exception_bounds is None:
        try:
            predicate(center)
        except Exception:
            raise ValueError(f"predicate always fails. An example traceback is shown above at {center}.")
    

    center, sign = binary_search(exception_predicate, bounds=exception_bounds, T=T, return_sign=True)
    at_center = predicate(center)
    return signed_band_search(center, at_center, sign)

