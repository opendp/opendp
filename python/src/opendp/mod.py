import ctypes
from typing import Union, Tuple, Callable

from opendp._lib import AnyMeasurement, AnyTransformation


class Measurement(ctypes.POINTER(AnyMeasurement)):
    """A differentially private unit of computation.
    A measurement contains a function and a privacy relation.
    The function releases a differentially-private release.
    The privacy relation maps from an input metric to an output measure.

    :example:

    >>> from opendp.mod import Measurement, enable_features
    >>> enable_features("contrib")
    ...
    >>> # create an instance of Measurement using a constructor from the meas module
    >>> from opendp.meas import make_base_geometric
    >>> base_geometric: Measurement = make_base_geometric(scale=2.)
    ...
    >>> # invoke the measurement (invoke and __call__ are equivalent)
    >>> base_geometric.invoke(100)  # -> 101   # doctest: +SKIP
    >>> base_geometric(100)  # -> 99           # doctest: +SKIP
    ...
    >>> # check the measurement's relation at
    >>> #     (1, 0.5): (AbsoluteDistance<u32>, MaxDivergence)
    >>> assert base_geometric.check(1, 0.5)
    ...
    >>> # chain with a transformation from the trans module
    >>> from opendp.trans import make_count
    >>> chained = (
    ...     make_count(TIA=int) >>
    ...     base_geometric
    ... )
    ...
    >>> # the resulting measurement has the same features
    >>> chained([1, 2, 3])  # -> 4     # doctest: +SKIP
    >>> # check the chained measurement's relation at
    >>> #     (1, 0.5): (SubstituteDistance, MaxDivergence)
    >>> assert chained.check(1, 0.5)
    """
    _type_ = AnyMeasurement

    def __call__(self, arg):
        from opendp.core import measurement_invoke
        return measurement_invoke(self, arg)

    def invoke(self, arg):
        """Create a differentially-private release with `arg`.
        
        :param arg: Input to the measurement.
        :return: differentially-private release
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import measurement_invoke
        return measurement_invoke(self, arg)

    def check(self, d_in, d_out, *, debug=False) -> bool:
        """Check if the measurement satisfies the privacy relation at `d_in`, `d_out`.
        
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

    @property
    def input_distance_type(self):
        """Retrieve the distance type of the input metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.
        
        :return: distance type
        """
        from opendp.core import measurement_input_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measurement_input_distance_type(self))

    @property
    def output_distance_type(self):
        """Retrieve the distance type of the output measure.
        This is the type that the budget is expressed in.
        
        :return: distance type
        """
        from opendp.typing import RuntimeType
        from opendp.core import measurement_output_distance_type
        return RuntimeType.parse(measurement_output_distance_type(self))

    @property
    def input_carrier_type(self):
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.
        
        :return: carrier type
        """
        from opendp.core import measurement_input_carrier_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measurement_input_carrier_type(self))

    def __del__(self):
        from opendp.core import _measurement_free
        _measurement_free(self)


class Transformation(ctypes.POINTER(AnyTransformation)):
    """A non-differentially private unit of computation.
    A transformation contains a function and a stability relation.
    The function maps from an input domain to an output domain.
    The stability relation maps from an input metric to an output metric.

    :example:

    >>> from opendp.mod import Transformation, enable_features
    >>> enable_features("contrib")
    ...
    >>> # create an instance of Transformation using a constructor from the trans module
    >>> from opendp.trans import make_count
    >>> count: Transformation = make_count(TIA=int)
    ...
    >>> # invoke the transformation (invoke and __call__ are equivalent)
    >>> count.invoke([1, 2, 3])  # -> 3  # doctest: +SKIP
    >>> count([1, 2, 3])  # -> 3         # doctest: +SKIP
    ...
    >>> # check the transformation's relation at
    >>> #     (1, 1): (SymmetricDistance, AbsoluteDistance<u32>)
    >>> assert count.check(1, 1)
    ...
    >>> # chain with more transformations from the trans module
    >>> from opendp.trans import make_split_lines, make_cast, make_impute_constant
    >>> chained = (
    ...     make_split_lines() >>
    ...     make_cast(TIA=str, TOA=int) >>
    ...     make_impute_constant(constant=0) >>
    ...     count
    ... )
    ...
    >>> # the resulting transformation has the same features
    >>> chained("1\\n2\\n3")  # -> 3 # doctest: +SKIP
    >>> assert chained.check(1, 1)  # both chained transformations were 1-stable
    """
    _type_ = AnyTransformation

    def invoke(self, arg):
        """Execute a non-differentially-private query with `arg`.

        :param arg: Input to the transformation.
        :return: non-differentially-private answer
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import transformation_invoke
        return transformation_invoke(self, arg)

    def __call__(self, arg):
        from opendp.core import transformation_invoke
        return transformation_invoke(self, arg)

    def check(self, d_in, d_out, *, debug=False):
        """Check if the transformation satisfies the stability relation at `d_in`, `d_out`.

        :param d_in: Distance in terms of the input metric.
        :param d_out: Distance in terms of the output metric.
        :param debug: Enable to raise Exceptions to help identify why the stability relation failed.
        :return: True if the relation passes. False if the relation failed.
        :rtype: bool
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import transformation_check

        if debug:
            return transformation_check(self, d_in, d_out)

        try:
            return transformation_check(self, d_in, d_out)
        except OpenDPException as err:
            if err.variant == "RelationDebug":
                return False
            raise

    def __rshift__(self, other: Union["Measurement", "Transformation"]):
        if isinstance(other, Measurement):
            from opendp.comb import make_chain_mt
            return make_chain_mt(other, self)

        if isinstance(other, Transformation):
            from opendp.comb import make_chain_tt
            return make_chain_tt(other, self)

        raise ValueError(f"rshift expected a measurement or transformation, got {other}")

    @property
    def input_distance_type(self):
        """Retrieve the distance type of the input metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.

        :return: distance type
        """
        from opendp.core import transformation_input_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(transformation_input_distance_type(self))

    @property
    def output_distance_type(self):
        """Retrieve the distance type of the output metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.

        :return: distance type
        """
        from opendp.core import transformation_output_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(transformation_output_distance_type(self))
    
    @property
    def input_carrier_type(self):
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.

        :return: carrier type
        """
        from opendp.core import transformation_input_carrier_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(transformation_input_carrier_type(self))

    def __del__(self):
        try:
            from opendp.core import _transformation_free
            _transformation_free(self)
        except ImportError:
            pass


class UnknownTypeException(Exception):
    pass


class OpenDPException(Exception):
    """General exception for errors originating from the underlying OpenDP library.
    The variant attribute corresponds to `one of the following variants <https://github.com/opendp/opendp/blob/53ec58d01762ca5ceee08590d7e7b725bbdafcf6/rust/opendp/src/error.rs#L46-L87>`_ and can be matched on.
    Error variants may change in library updates.

    .. todo:: Link to generated rust documentation for ErrorVariant.
    """
    def __init__(self, variant: str, message: str = None, inner_traceback: str = None):
        self.variant = variant
        self.message = message
        self.inner_traceback = inner_traceback

    def __str__(self) -> str:
        response = self.variant
        if self.message:
            response += f'("{self.message}")'
        if self.inner_traceback:
            response += "\n" + '\n'.join('\t' + line for line in self.inner_traceback.split('\n'))
        return response


GLOBAL_FEATURES = set()


def enable_features(*features: str) -> None:
    GLOBAL_FEATURES.update(set(features))


def disable_features(*features: str) -> None:
    GLOBAL_FEATURES.difference_update(set(features))


def assert_features(*features: str) -> None:
    for feature in features:
        assert feature in GLOBAL_FEATURES, f"Attempted to use function that requires {feature}, but {feature} is not enabled. See https://github.com/opendp/opendp/discussions/304, then call enable_features(\"{feature}\")"


def binary_search_chain(
        make_chain: Callable[[Union[float, int]], Union[Transformation, Measurement]],
        d_in, d_out,
        bounds: Union[Tuple[float, float], Tuple[int, int]] = None,
        tolerance=None) -> Union[Transformation, Measurement]:
    """Optimizes a parameterized chain `make_chain` within float or integer `bounds`,
    subject to the chained relation being (`d_in`, `d_out`)-close.

    `bounds` defaults to (0., MAX_FINITE_FLOAT).
    If `bounds` are float, `tolerance` defaults to 1e-8.

    See `binary_search_param` to retrieve the discovered parameter instead of the complete computation chain.

    :param make_chain: a unary function that maps from a number to a Transformation or Measurement
    :param d_in: desired input distance of the computation chain
    :param d_out: desired output distance of the computation chain
    :param bounds: a 2-tuple of the lower and upper bounds to the input of `make_chain`
    :param tolerance: the discovered parameter differs by at most `tolerance` from the ideal parameter
    :return: a chain parameterized at the nearest passing value to the decision point of the relation
    :raises AssertionError: if the arguments are ill-formed (type issues, decision boundary not within `bounds`)


    :example:

    >>> from opendp.mod import binary_search_chain, enable_features
    >>> from opendp.trans import make_clamp, make_bounded_resize, make_sized_bounded_mean
    >>> from opendp.meas import make_base_laplace
    >>> enable_features("floating-point", "contrib")
    ...
    >>> # The majority of the chain only needs to be defined once.
    >>> pre = (
    ...     make_clamp(bounds=(0., 1.)) >>
    ...     make_bounded_resize(size=10, bounds=(0., 1.), constant=0.) >>
    ...     make_sized_bounded_mean(size=10, bounds=(0., 1.))
    ... )
    ...
    >>> # Find a value in `bounds` that produces a (`d_in`, `d_out`)-chain within `tolerance` of the decision boundary.
    >>> # The lambda function returns the complete computation chain when given a single numeric parameter.
    >>> chain = binary_search_chain(lambda s: pre >> make_base_laplace(scale=s), d_in=1, d_out=1.)
    ...
    >>> # The resulting computation chain is always (`d_in`, `d_out`)-close, but we can still double-check:
    >>> assert chain.check(1, 1.)
    """
    return make_chain(binary_search_param(make_chain, d_in, d_out, bounds, tolerance))


def binary_search_param(
        make_chain: Callable[[Union[float, int]], Union[Transformation, Measurement]],
        d_in, d_out,
        bounds: Union[Tuple[float, float], Tuple[int, int]] = None,
        tolerance=None) -> Union[float, int]:
    """Optimizes a parameterized chain `make_chain` within float or integer `bounds`,
    subject to the chained relation being (`d_in`, `d_out`)-close.

    `bounds` defaults to (0., MAX_FINITE_FLOAT).
    If `bounds` are float, `tolerance` defaults to 1e-8.

    :example:

    >>> from opendp.mod import binary_search_param, enable_features
    >>> from opendp.meas import make_base_laplace
    ...
    >>> # Find a value in `bounds` that produces a (`d_in`, `d_out`)-chain within `tolerance` of the decision boundary.
    >>> # The first argument is any function that returns your complete computation chain
    >>> #     when passed a single numeric parameter.
    >>> scale = binary_search_param(make_base_laplace, d_in=0.1, d_out=1.)
    >>> # The discovered scale differs by at most `tolerance` from the ideal scale (0.1).
    >>> assert scale - 0.1 < 1e-8
    >>> # Constructing the same chain with the discovered parameter will always be (0.1, 1.)-close.
    >>> assert make_base_laplace(scale).check(0.1, 1.)

    :param make_chain: a unary function that maps from a number to a Transformation or Measurement
    :param d_in: desired input distance of the computation chain
    :param d_out: desired output distance of the computation chain
    :param bounds: a 2-tuple of the lower and upper bounds to the input of `make_chain`
    :param tolerance: the discovered parameter differs by at most `tolerance` from the ideal parameter
    :return: the nearest passing value to the decision point of the relation
    :raises AssertionError: if the arguments are ill-formed (type issues, decision boundary not within `bounds`)
    """
    if bounds is None:
        import sys
        bounds = (0., sys.float_info.max)
    return binary_search(lambda param: make_chain(param).check(d_in, d_out), bounds, tolerance)


def binary_search(
        predicate: Callable[[Union[float, int]], bool],
        bounds: Union[Tuple[float, float], Tuple[int, int]],
        tolerance=None):
    """Find the closest passing value to the decision boundary of `predicate` within float or integer `bounds`.

    If `bounds` are float, `tolerance` defaults to 1e-8.

    :example:

    >>> from opendp.mod import binary_search
    >>> # Integer binary search
    >>> assert binary_search(lambda x: x > 5, bounds=(0, 10)) == 6
    >>> assert binary_search(lambda x: x < 5, bounds=(0, 10)) == 4
    >>> # Float binary search
    >>> assert binary_search(lambda x: x > 5., bounds=(0., 10.)) - 5. < 1e-8
    >>> assert binary_search(lambda x: x > 5., bounds=(0., 10.)) - 5. > -1e-8

    :param predicate: a monotonic unary function from a number to a boolean
    :param bounds: a 2-tuple of the lower and upper bounds to the input of `predicate`
    :param tolerance: the discovered parameter differs by at most `tolerance` from the ideal parameter
    :return: the discovered parameter within the bounds
    :raises AssertionError: if the arguments are ill-formed (type issues, decision boundary not within `bounds`)
    """
    assert len(set(map(type, bounds))) == 1, "bounds must share the same type"
    lower, upper = sorted(bounds)

    maximize = predicate(lower)  # if the lower bound passes, we should maximize
    minimize = predicate(upper)  # if the upper bound passes, we should minimize
    assert maximize != minimize, "the decision boundary of the predicate is outside the bounds"

    if isinstance(lower, float):
        tolerance = 1.0e-8 if tolerance is None else tolerance
        half = lambda x: x / 2.
    elif isinstance(lower, int):
        tolerance = tolerance or 1  # the lower and upper bounds never meet due to int truncation
        half = lambda x: x // 2
    else:
        raise AssertionError("bounds must be either float or int")

    while upper - lower > tolerance:
        mid = lower + half(upper - lower)  # avoid overflow
        if predicate(mid) == minimize:
            upper = mid
        else:
            lower = mid

    # one bound is always false, the other true. Return the truthy bound
    return upper if minimize else lower
