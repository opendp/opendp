import ctypes
from typing import Union, Tuple, Any, Callable

from opendp._lib import AnyMeasurement, AnyTransformation


class Measurement(ctypes.POINTER(AnyMeasurement)):
    """A differentially private unit of computation.
    A measurement contains a function and a privacy relation.
    The function releases a differentially-private release.
    The privacy relation maps from an input metric to an output measure.

    :example:

    >>> from opendp.mod import Measurement
    >>>
    >>> # create an instance of Measurement using a constructor from the meas module
    >>> from opendp.meas import make_base_geometric
    >>> base_geometric: Measurement = make_base_geometric(scale=2., lower=0, upper=20)
    >>>
    >>> # invoke the measurement (invoke and __call__ are equivalent)
    >>> base_geometric.invoke(100)  # -> 101
    >>> base_geometric(100)  # -> 99
    >>>
    >>> # check the measurement's relation at
    >>> #     (1, 0.5): (AbsoluteDistance<u32>, MaxDivergence)
    >>> assert base_geometric.check(1, 0.5)
    >>>
    >>> # chain with a transformation from the trans module
    >>> from opendp.trans import make_count
    >>> from opendp.typing import SubstituteDistance
    >>> chained = (
    >>>     make_count(MI=SubstituteDistance, TI=int) >>
    >>>     base_geometric
    >>> )
    >>>
    >>> # the resulting measurement has the same features
    >>> chained([1, 2, 3])  # -> 4
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

    >>> from opendp.mod import Transformation
    >>>
    >>> # create an instance of Transformation using a constructor from the trans module
    >>> from opendp.trans import make_count
    >>> count: Transformation = make_count(MI=SymmetricDistance, TI=int)
    >>>
    >>> # invoke the transformation (invoke and __call__ are equivalent)
    >>> count.invoke([1, 2, 3])  # -> 3
    >>> count([1, 2, 3])  # -> 3
    >>>
    >>> # check the transformation's relation at
    >>> #     (1, 1): (SymmetricDistance, AbsoluteDistance<u32>)
    >>> assert count.check(1, 1)
    >>>
    >>> # chain with more transformations from the trans module
    >>> from opendp.trans import make_split_lines, make_cast
    >>> from opendp.typing import SymmetricDistance
    >>> chained = (
    >>>     make_split_lines(M=SymmetricDistance) >>
    >>>     make_cast(M=SymmetricDistance, TI=str, TO=int) >>
    >>>     count
    >>> )
    >>>
    >>> # the resulting transformation has the same features
    >>> chained("1\\n2\\n3")  # -> 3
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
        assert feature in GLOBAL_FEATURES, f"Attempted to use function that requires {feature}, but {feature} is not enabled. Check the documentation for the feature, then call enable_features(\"{feature}\")"


def binary_search_chain(
        make_chain: Callable[[Union[float, int]], Union[Transformation, Measurement]],
        bounds: Union[Tuple[float, float], Tuple[int, int]],
        d_in, d_out, tolerance=1e-8) -> Union[Transformation, Measurement]:
    """Optimizes a parameterized chain `make_chain` subject to float or integer `bounds`,
    subject to the chained relation being (`d_in`, `d_out`)-close.

    :param make_chain: a function with one parameter that constructs a Transformation or Measurement
    :param bounds: a 2-tuple of the parameter's lower and upper bounds
    :param d_in: desired input distance of the computation chain
    :param d_out: desired output distance of the computation chain
    :param tolerance: the discovered parameter differs by at most `tolerance` from the ideal parameter
    :return: A chain parameterized at the nearest passing value to the decision point of the relation.
    """
    return make_chain(binary_search_param(make_chain, bounds, d_in, d_out, tolerance))


def binary_search_param(
        make_chain: Callable[[Union[float, int]], Union[Transformation, Measurement]],
        bounds: Union[Tuple[float, float], Tuple[int, int]],
        d_in, d_out, tolerance=1e-8) -> Union[float, int]:
    """Optimizes a parameterized chain `make_chain` subject to float or integer `bounds`,
    subject to the chained relation being (`d_in`, `d_out`)-close.

    :param make_chain: a function with one parameter that constructs a Transformation or Measurement
    :param bounds: a 2-tuple of the parameter's lower and upper bounds
    :param d_in: desired input distance of the computation chain
    :param d_out: desired output distance of the computation chain
    :param tolerance: the discovered parameter differs by at most `tolerance` from the ideal parameter
    :return: The nearest passing value to the decision point of the relation.
    """
    return binary_search(lambda p: make_chain(p).check(d_in, d_out), bounds, tolerance)


def binary_search(
        predicate: Callable[[Union[float, int]], bool],
        bounds: Union[Tuple[float, float], Tuple[int, int]],
        tolerance=1e-8):
    """Find the closest value to the `predicate`'s decision point within float or integer `bounds`.

    :param predicate: a function with one parameter that returns a boolean
    :param bounds: the parameter's lower and upper bounds
    :param tolerance: the discovered float parameter differs by at most `tolerance` from the ideal float parameter
    :return: the discovered parameter within the bounds
    """
    assert len(bounds) == 2, "lower and upper bound must be provided"
    assert len(set(map(type, bounds))) == 1, "bounds must share the same type"
    lower, upper = sorted(bounds)

    maximize = predicate(lower)
    minimize = predicate(upper)

    assert maximize != minimize, "the decision point of the predicate is outside the bounds"

    if isinstance(lower, int):
        while upper != lower:
            mid = lower + (upper - lower) // 2

            if predicate(mid) == minimize:
                upper = mid
            else:
                lower = mid + 1

        # make sure the search terminates on a successful predicate
        if not predicate(lower):
            lower += 1 if minimize else -1
        return lower

    elif isinstance(lower, float):
        assert isinstance(tolerance, float), 'tolerance must be a float'
        while upper - lower > tolerance:
            mid = lower + (upper - lower) / 2.

            if predicate(mid) == minimize:
                upper = mid
            else:
                lower = mid

        # make sure the search terminates on a successful predicate
        return upper if minimize else lower

    else:
        raise ValueError("bounds must be either float or int")
