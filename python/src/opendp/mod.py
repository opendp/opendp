'''
The ``mod`` module provides the classes which implement the
`OpenDP Programming Framework <../../api/user-guide/programming-framework/index.html>`_,
as well as utilities for enabling features and finding parameter values.

The classes here correspond to other top-level modules: For example,
instances of :py:class:`opendp.mod.Domain` are either inputs or outputs for functions in :py:mod:`opendp.domains`.
'''
from __future__ import annotations
import ctypes
from dataclasses import asdict
from typing import Any, Literal, Sequence, Type, TypeVar, Union, Callable, Optional, overload, TYPE_CHECKING, cast
import importlib
import json

from opendp._lib import AnyMeasurement, AnyTransformation, AnyDomain, AnyMetric, AnyMeasure, AnyFunction, AnyOdometer, import_optional_dependency, get_opendp_version


# https://mypy.readthedocs.io/en/stable/runtime_troubles.html#import-cycles
if TYPE_CHECKING:
    from opendp.typing import RuntimeType # pragma: no cover


__all__ = [
    'Measurement',
    'Transformation',
    'Odometer',
    'Queryable',
    'OdometerQueryable',
    'Function',
    'Domain',
    'AtomDomain',
    'OptionDomain',
    'VectorDomain',
    'SeriesDomain',
    'LazyFrameDomain',
    'ExtrinsicDomain',
    'Metric',
    'SymmetricIdDistance',
    'ChangeOneIdDistance',
    'FrameDistance',
    'Measure',
    'ApproximateDivergence',
    'PrivacyProfile',
    '_PartialConstructor',
    'UnknownTypeException',
    'OpenDPException',
    'GLOBAL_FEATURES',
    'enable_features',
    'disable_features',
    'assert_features',
    'binary_search_chain',
    'binary_search_param',
    'binary_search',
    'exponential_bounds_search',
    'serialize',
    'deserialize',
    '__version__',
]

class Measurement(ctypes.POINTER(AnyMeasurement)): # type: ignore[misc]
    """A differentially private unit of computation.
    A measurement contains a function and a privacy relation.
    The function releases a differentially-private release.
    The privacy relation maps from an input metric to an output measure.

    See the `Measurement <../../api/user-guide/programming-framework/core-structures.html#measurement>`_
    section in the Programming Framework docs for more context.

    Functions for creating measurements are in :py:mod:`opendp.measurements`.

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    >>> # create an instance of Measurement using a constructor from the meas module
    >>> laplace = dp.m.make_laplace(
    ...     dp.atom_domain(T=int), dp.absolute_distance(T=int),
    ...     scale=2.)
    >>> laplace
    Measurement(
        input_domain   = AtomDomain(T=i32),
        input_metric   = AbsoluteDistance(i32),
        output_measure = MaxDivergence)

    >>> # invoke the measurement (invoke and __call__ are equivalent)
    >>> print('explicit: ', laplace.invoke(100))  # -> 101   # doctest: +ELLIPSIS
    explicit: ...
    >>> print('concise: ', laplace(100))  # -> 99            # doctest: +ELLIPSIS
    concise: ...
    >>> # check the measurement's relation at
    >>> #     (1, 0.5): (AbsoluteDistance<u32>, MaxDivergence)
    >>> assert laplace.check(1, 0.5)

    >>> # chain with a transformation from the trans module
    >>> chained = (
    ...     (dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance()) >>
    ...     dp.t.then_count() >>
    ...     laplace
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
        """Map an input distance `d_in` to an output distance.
        
        :param d_in: Input distance
        """
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
                return False # pragma: no cover
            raise

    def __rshift__(self, other: Union["Function", "Transformation", Callable]) -> "Measurement":
        if isinstance(other, Transformation):
            other = other.function

        if not isinstance(other, Function):
            if not callable(other):
                raise ValueError(f'Expected a callable instead of {other}')  # pragma: no cover
            from opendp.core import new_function
            other = new_function(other, TO="ExtrinsicObject")

        from opendp.combinators import make_chain_pm
        return make_chain_pm(other, self)

    @property
    def input_domain(self) -> "Domain":
        '''
        Input domain of measurement
        '''
        from opendp.core import measurement_input_domain
        return measurement_input_domain(self)
    
    @property
    def input_metric(self) -> "Metric":
        '''
        Input metric of measurement
        '''
        from opendp.core import measurement_input_metric
        return measurement_input_metric(self)

    @property
    def input_space(self) -> tuple["Domain", "Metric"]:
        '''
        Input space of measurement
        '''
        return self.input_domain, self.input_metric
    
    @property
    def output_measure(self) -> "Measure":
        '''
        Output measure of measurement
        '''
        from opendp.core import measurement_output_measure
        return measurement_output_measure(self)
    
    @property
    def function(self) -> "Function":
        '''
        Function of measurement
        '''
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

    def __del__(self):
        try:
            from opendp.core import _measurement_free
            _measurement_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass
    
    def __repr__(self) -> str:
        return f"""Measurement(
    input_domain   = {self.input_domain},
    input_metric   = {self.input_metric},
    output_measure = {self.output_measure})"""

    def __iter__(self):
        # this overrides the implementation of __iter__ on POINTER, 
        # which yields infinitely on zero-sized types
        raise ValueError("Measurement does not support iteration")
    

class Odometer(ctypes.POINTER(AnyOdometer)): # type: ignore[misc]
    """A differentially private unit of computation with no privacy limit.
    An Odometer can be invoked with a dataset to return an OdometerQueryable.

    Differentially private queries (measurements) may be passed to the queryable,
    and ``.privacy_loss(d_in)`` can be called to check the current privacy usage.

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")
    ...
    >>> # create a measurement that responds honestly with probability 0.6
    >>> meas_rr = dp.m.make_randomized_response_bool(prob=0.6)
    ...
    >>> # create an Odometer representing a fully adaptive compositor
    >>> odometer = dp.c.make_fully_adaptive_composition(
    ...     input_domain=meas_rr.input_domain,
    ...     input_metric=meas_rr.input_metric,
    ...     output_measure=meas_rr.output_measure,
    ... )
    ...
    >>> # invoke the odometer to get a queryable
    >>> data = True # a trivial boolean dataset
    >>> qbl_comp = odometer(data)
    ...
    >>> # evaluate the queryable (eval and __call__ are equivalent)
    >>> print("Release:", qbl_comp(meas_rr))  # doctest: +ELLIPSIS
    Release: ...
    >>> # odometers can be repeatedly invoked without any limit on the privacy loss
    >>> print("Release:", qbl_comp(meas_rr))  # doctest: +ELLIPSIS
    Release: ...
    >>> # The odometer's privacy consumption (in terms of ε) can be checked at any time.
    >>> # Input dataset may differ by discrete distance 1.
    >>> qbl_comp.privacy_loss(1)
    0.8109302162163288
    """
    _type_ = AnyOdometer

    def __call__(self, arg):
        from opendp.core import odometer_invoke
        return odometer_invoke(self, arg)

    def invoke(self, arg):
        """Create a differentially-private release with `arg`.

        If `self` is (d_in, d_out)-close, then each invocation of this function is a d_out-DP release. 
        
        :param arg: Input to the measurement.
        :return: differentially-private release
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.core import odometer_invoke
        return odometer_invoke(self, arg)

    @property
    def input_domain(self) -> "Domain":
        '''
        Input domain of odometer
        '''
        from opendp.core import odometer_input_domain
        return odometer_input_domain(self)
    
    @property
    def input_metric(self) -> "Metric":
        '''
        Input metric of odometer
        '''
        from opendp.core import odometer_input_metric
        return odometer_input_metric(self)
    
    @property
    def input_space(self) -> tuple["Domain", "Metric"]:
        '''
        Input domain and metric of odometer
        '''
        return self.input_domain, self.input_metric
    
    @property
    def output_measure(self) -> "Measure":
        '''
        Output measure of odometer
        '''
        from opendp.core import odometer_output_measure
        return odometer_output_measure(self)
    
    @property
    def input_distance_type(self) -> Union["RuntimeType", str]:
        """Retrieve the distance type of the input metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.
        
        :return: distance type
        """
        return self.input_metric.distance_type

    @property
    def output_distance_type(self) -> Union["RuntimeType", str]:
        """Retrieve the distance type of the output measure.
        This is the type that the budget is expressed in.
        
        :return: distance type
        """
        return self.output_measure.distance_type

    @property
    def input_carrier_type(self) -> Union["RuntimeType", str]:
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.
        
        :return: carrier type
        """
        return self.input_domain.carrier_type

    def __del__(self):
        try:
            from opendp.core import _odometer_free
            _odometer_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # ImportError: sys.meta_path is None, Python is likely shutting down
            # TypeError: similar setting as above
            pass
    
    def __repr__(self) -> str:
        return f"""Odometer(
    input_domain   = {self.input_domain},
    input_metric   = {self.input_metric},
    output_measure = {self.output_measure})"""

    def __iter__(self):
        # this overrides the implementation of __iter__ on POINTER, 
        # which yields infinitely on zero-sized types
        raise ValueError("Odometer does not support iteration")


class Transformation(ctypes.POINTER(AnyTransformation)): # type: ignore[misc]
    """A non-differentially private unit of computation.
    A transformation contains a function and a stability relation.
    The function maps from an input domain to an output domain.
    The stability relation maps from an input metric to an output metric.

    See the `Transformation <../../api/user-guide/programming-framework/core-structures.html#transformation>`_
    section in the Programming Framework docs for more context.

    Functions for creating transformations are in :py:mod:`opendp.transformations`.

    :example:

    >>> import opendp.prelude as dp
    >>> dp.enable_features("contrib")

    >>> # create an instance of Transformation using a constructor from the trans module
    >>> input_space = (dp.vector_domain(dp.atom_domain(T=int)), dp.symmetric_distance())
    >>> count = input_space >> dp.t.then_count()
    >>> count
    Transformation(
        input_domain   = VectorDomain(AtomDomain(T=i32)),
        output_domain  = AtomDomain(T=i32),
        input_metric   = SymmetricDistance(),
        output_metric  = AbsoluteDistance(i32))

    >>> count.input_space
    (VectorDomain(AtomDomain(T=i32)), SymmetricDistance())
    
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
        from opendp.core import transformation_invoke
        return transformation_invoke(self, arg) 

    def __call__(self, arg):
        from opendp.core import transformation_invoke
        return transformation_invoke(self, arg)

    def map(self, d_in):
        """Map an input distance `d_in` to an output distance.
        
        :param d_in: Input distance. An upper bound on how far apart neighboring datasets can be with respect to the input metric
        """
        from opendp.core import transformation_map
        return transformation_map(self, d_in)

    def check(self, d_in, d_out, *, debug=False) -> bool:
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
            return transformation_check(self, d_in, d_out)

        try:
            return transformation_check(self, d_in, d_out)
        except OpenDPException as err: # pragma: no cover
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
    def __rshift__(self, other: "Odometer") -> "Odometer":
        ...

    @overload
    def __rshift__(self, other: "_PartialConstructor") -> Union["Transformation", "Measurement", "Odometer"]:
        ...

    def __rshift__(self, other: Union["Measurement", "Transformation", "_PartialConstructor"]) -> Union["Measurement", "Transformation", "_PartialConstructor", "PartialChain"]:  # type: ignore[name-defined] # noqa F821
        if isinstance(other, Measurement):
            from opendp.combinators import make_chain_mt
            return make_chain_mt(other, self)

        if isinstance(other, Transformation):
            from opendp.combinators import make_chain_tt
            return make_chain_tt(other, self)
        
        if isinstance(other, _PartialConstructor):
            return self >> other(self.output_domain, self.output_metric) # type: ignore[call-arg]

        from opendp.context import PartialChain
        if isinstance(other, PartialChain):
            return PartialChain(lambda x: self >> other.partial(x))

        raise ValueError(f"rshift expected a measurement or transformation, got {other}")  # pragma: no cover


    @property
    def input_domain(self) -> "Domain":
        '''
        Input domain of transformation
        '''
        from opendp.core import transformation_input_domain
        return transformation_input_domain(self)
    

    @property
    def output_domain(self) -> "Domain":
        '''
        Output domain of transformation
        '''
        from opendp.core import transformation_output_domain
        return transformation_output_domain(self)
    

    @property
    def input_metric(self) -> "Metric":
        '''
        Input metric of transformation
        '''
        from opendp.core import transformation_input_metric
        return transformation_input_metric(self)
    
    @property
    def output_metric(self) -> "Metric":
        '''
        Ouput metric of transformation
        '''
        from opendp.core import transformation_output_metric
        return transformation_output_metric(self)
    
    @property
    def input_space(self) -> tuple["Domain", "Metric"]:
        '''
        Input space of transformation
        '''
        return self.input_domain, self.input_metric
    
    @property
    def output_space(self) -> tuple["Domain", "Metric"]:
        '''
        Output space of transformation
        '''
        return self.output_domain, self.output_metric
    
    @property
    def function(self) -> "Function":
        '''
        Function of transformation
        '''
        from opendp.core import transformation_function
        return transformation_function(self)

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

    def __del__(self):
        try:
            from opendp.core import _transformation_free
            _transformation_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __repr__(self) -> str:
        return f"""Transformation(
    input_domain   = {self.input_domain},
    output_domain  = {self.output_domain},
    input_metric   = {self.input_metric},
    output_metric  = {self.output_metric})"""
    
    def __iter__(self):
        raise ValueError("Transformation does not support iteration")


Transformation = cast(Type[Transformation], Transformation) # type: ignore[misc]

class Queryable:
    '''
    Queryables are used for interactive mechanisms like :ref:`adaptive composition <adaptive-composition>`.

    Queryables can be created with :py:func:`make_adaptive_composition <opendp.combinators.make_adaptive_composition>`
    or :py:func:`new_queryable <opendp.core.new_queryable>`.
    '''
    def __init__(self, value, query_type):
        self.value = value
        self.query_type = query_type

    def __call__(self, query):
        from opendp.core import queryable_eval
        return queryable_eval(self.value, query)
    
    def __repr__(self) -> str:
        return f"Queryable(Q={self.query_type})"

class OdometerQueryable:
    '''
    Odometer Queryables are used for instances of odometers like :ref:`fully adaptive composition <fully-adaptive-composition>`.

    Can be created via :py:func:`make_fully_adaptive_composition <opendp.combinators.make_fully_adaptive_composition>`.
    '''
    def __init__(self, value):
        self.value = value

    def __call__(self, query):
        from opendp.core import odometer_queryable_invoke
        return odometer_queryable_invoke(self.value, query)
    
    def invoke(self, query):
        from opendp.core import odometer_queryable_invoke
        return odometer_queryable_invoke(self.value, query)
    
    def privacy_loss(self, d_in):
        from opendp.core import odometer_queryable_privacy_loss
        return odometer_queryable_privacy_loss(self.value, d_in)

    def __repr__(self) -> str:
        from opendp.core import odometer_queryable_invoke_type, odometer_queryable_privacy_loss_type
        from opendp.typing import RuntimeType
        Q = RuntimeType.parse(odometer_queryable_invoke_type(self.value))
        QB = RuntimeType.parse(odometer_queryable_privacy_loss_type(self.value))
        return f"OdometerQueryable(Q={Q}, QB={QB})"


class Function(ctypes.POINTER(AnyFunction)): # type: ignore[misc]
    '''
    See the `Function <../../api/user-guide/programming-framework/supporting-elements.html#function>`_
    section in the Programming Framework docs for more context.
    '''
    _type_ = AnyFunction

    def __call__(self, arg):
        from opendp.core import function_eval
        return function_eval(self, arg)
    
    def __del__(self):
        try:
            from opendp.core import _function_free
            _function_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __iter__(self):
        raise ValueError("Function does not support iteration")


D = TypeVar("D")

class Domain(ctypes.POINTER(AnyDomain)): # type: ignore[misc]
    '''
    See the `Domain <../../api/user-guide/programming-framework/supporting-elements.html#domain>`_
    section in the Programming Framework docs for more context.

    Functions for creating domains are in :py:mod:`opendp.domains`.
    '''
    
    # for documentation see https://docs.python.org/3/library/ctypes.html#ctypes._Pointer._type_
    _type_ = AnyDomain

    def member(self, val) -> bool:
        '''
        Check if ``val`` is a member of the domain.
        
        :param val: a value to be checked for membership in `self`
        '''
        try:
            from opendp.domains import _member
            return _member(self, val)
        except Exception as e:
            from warnings import warn
            warn(f'Value ({val}) does not belong to carrier type of {self}. Details: {e}')
            return False


    @property
    def type(self) -> Union["RuntimeType", str]:
        '''
        Type of domain
        '''
        from opendp.domains import domain_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(domain_type(self))
    
    @property
    def carrier_type(self) -> Union["RuntimeType", str]:
        '''
        Carrier type of domain
        '''
        from opendp.domains import domain_carrier_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(domain_carrier_type(self))
    
    def __repr__(self) -> str:
        from opendp.domains import domain_debug
        return domain_debug(self)
    
    def __del__(self):
        try:
            from opendp.domains import _domain_free
            _domain_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass
    
    def __eq__(self, other) -> bool:
        from opendp.domains import _domain_equal

        if not isinstance(other, Domain):
            return False
        
        return _domain_equal(self, other)
    
    def __hash__(self) -> int:
        return hash(str(self))
    
    def __iter__(self):
        raise ValueError("Domain does not support iteration")
    
    def cast(self, type_: Type[D]) -> D:
        """Retrieve the descriptor as the prescribed type, or error."""
        if not (
            isinstance(self, ExtrinsicDomain)
            and isinstance(descriptor := self.descriptor, type_)
        ):
            raise ValueError(f"domain descriptor must be a {type_.__name__}")
        return descriptor


class AtomDomain(Domain):
    '''The domain of all values of a given atomic type.

    Create an instance of this domain with :py:func:`opendp.domains.atom_domain`.

    If bounds are set, then the domain is restricted to the bounds.
    If nullable is set, then null value(s) are included in the domain.
    '''

    _type_ = AnyDomain
    
    @property
    def bounds(self) -> tuple[float, float]:
        '''Bounds of the domain, if they exist'''
        from opendp.domains import _atom_domain_get_bounds_closed
        return _atom_domain_get_bounds_closed(self)
    
    @property
    def nan(self) -> bool:
        """Whether the domain includes NaN values
        
        Only relevant when the carrier type is a floating point type.
        All other types will always return ``False``.
        """
        from opendp.domains import _atom_domain_nan
        return _atom_domain_nan(self)
    

class OptionDomain(Domain):
    '''A domain whose members are either members of the ``element_domain``, or ``None``.

    Create an instance of this domain with :py:func:`opendp.domains.option_domain`.

    The element domain is the domain of non-null values.
    '''

    _type_ = AnyDomain

    @property
    def element_domain(self) -> Domain:
        '''Domain of non-null values'''
        from opendp.domains import _option_domain_get_element_domain
        return _option_domain_get_element_domain(self)


class VectorDomain(Domain):
    '''``VectorDomain`` describes the domain of all vectors whose elements are members of a given domain.
    
    Create an instance of this domain with :py:func:`opendp.domains.vector_domain`.
    '''
    _type_ = AnyDomain
    
    @property
    def element_domain(self) -> Domain:
        '''Domain of elements in the vector'''
        from opendp.domains import _vector_domain_get_element_domain
        return _vector_domain_get_element_domain(self)
    
    @property
    def size(self) -> Optional[int]:
        '''Size of vectors in the domain, if it is fixed'''
        from opendp.domains import _vector_domain_get_size
        return _vector_domain_get_size(self)


class SeriesDomain(Domain):
    '''``SeriesDomain`` describes the domain of all polars Series.
    
    Create an instance of this domain with :py:func:`opendp.domains.series_domain`.
    '''
    _type_ = AnyDomain

    @property
    def name(self) -> str:
        '''Name of series in the domain'''
        from opendp.domains import _series_domain_get_name
        return _series_domain_get_name(self)
    
    @property
    def element_domain(self) -> Domain:
        '''Domain of non-null elements in the series'''
        from opendp.domains import _series_domain_get_element_domain
        return _series_domain_get_element_domain(self)
    
    @property
    def nullable(self) -> bool:
        '''Whether series in the domain may include null values'''
        from opendp.domains import _series_domain_get_nullable
        return _series_domain_get_nullable(self)
    
class LazyFrameDomain(Domain):
    '''``LazyFrameDomain`` describes the domain of all polars LazyFrames.
    
    Create an instance of this domain with :py:func:`opendp.domains.lazyframe_domain`.
    '''
    _type_ = AnyDomain

    @property
    def columns(self) -> list[str]:
        '''List of column names in the frame'''
        from opendp.domains import _lazyframe_domain_get_columns
        return _lazyframe_domain_get_columns(self)

    def get_series_domain(self, name: str) -> SeriesDomain:
        '''Retrieve the series domain with the given name'''
        from opendp.domains import _lazyframe_domain_get_series_domain
        return _lazyframe_domain_get_series_domain(self, name)
    
    def get_margin(self, by: Sequence[Any]):
        '''Get the margin descriptor of the frame when grouped by the given columns'''
        from opendp.domains import _lazyframe_domain_get_margin
        from opendp._convert import _check_polars_by
        _check_polars_by(by)

        return _lazyframe_domain_get_margin(self, by)


class ExtrinsicDomain(Domain):
    '''A user-defined domain.'''

    _type_ = AnyDomain
        
    @property
    def descriptor(self) -> Any:
        '''
        Descriptor of domain. Used to retrieve the descriptor associated with domains defined in Python 
        '''
        from opendp.domains import _extrinsic_domain_descriptor
        return _extrinsic_domain_descriptor(self)


class Metric(ctypes.POINTER(AnyMetric)): # type: ignore[misc]
    '''
    See the `Metric <../../api/user-guide/programming-framework/supporting-elements.html#metric>`_
    section in the Programming Framework docs for more context.

    Functions for creating metrics are in :py:mod:`opendp.metrics`.
    '''
    _type_ = AnyMetric

    @property
    def type(self):
        '''
        Type of metric
        '''
        from opendp.metrics import metric_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(metric_type(self))
    
    @property
    def distance_type(self) -> Union["RuntimeType", str]:
        '''
        Distance type of metric
        '''
        from opendp.metrics import metric_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(metric_distance_type(self))

    def __repr__(self) -> str:
        from opendp.metrics import metric_debug
        return metric_debug(self)
    
    def __del__(self):
        try:
            from opendp.metrics import _metric_free
            _metric_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass
    
    def __eq__(self, other) -> bool:
        from opendp.metrics import _metric_equal

        if not isinstance(other, Metric):
            return False
        
        return _metric_equal(self, other)
    
    def __hash__(self) -> int:
        return hash(str(self))
    
    def __iter__(self):
        raise ValueError("Metric does not support iteration")
    
    def cast(self, type_: Type[D]) -> D:
        """Retrieve the descriptor as the prescribed type, or error."""
        if not (
            isinstance(self, ExtrinsicDistance)
            and isinstance(descriptor := self.descriptor, type_)
        ):
            raise ValueError(f"metric descriptor must be a {type_.__name__}, found {self}")
        return descriptor


class ExtrinsicDistance(Metric):
    '''A user-defined metric.'''

    _type_ = AnyMetric
        
    @property
    def descriptor(self) -> Any:
        '''
        Descriptor of domain. Used to retrieve the descriptor associated with domains defined in Python 
        '''
        from opendp.metrics import _extrinsic_metric_descriptor
        return _extrinsic_metric_descriptor(self)


class FrameDistance(Metric):
    '''``FrameDistance`` is a higher-order metric that contains multiple distance bounds for different groupings of data.'''

    _type_ = AnyMetric
    
    @property
    def inner_metric(self) -> Metric:
        '''Bounds of the domain, if they exist'''
        from opendp.metrics import _frame_distance_get_inner_metric
        return _frame_distance_get_inner_metric(self)
    
class SymmetricIdDistance(Metric):
    '''``SymmetricIdDistance`` is a metric for measuring the distance between the identifiers of two datasets.
    
    The metric counts the number of identifiers that must be added or removed to make the two datasets equal.
    '''

    _type_ = AnyMetric
    
    @property
    def identifier(self):
        '''The name of the column storing identifiers'''
        from opendp.metrics import _symmetric_id_distance_get_identifier
        return _symmetric_id_distance_get_identifier(self)
    

class ChangeOneIdDistance(Metric):
    '''``ChangeOneIdDistance`` is a metric for measuring the distance between the identifiers of two datasets.
    
    The metric counts the number of identifiers that must be changed to make the two datasets equal.
    '''

    _type_ = AnyMetric
    
    @property
    def identifier(self):
        '''The name of the column storing identifiers'''
        from opendp.metrics import _change_one_id_distance_get_identifier
        return _change_one_id_distance_get_identifier(self)
    
class Measure(ctypes.POINTER(AnyMeasure)): # type: ignore[misc]
    '''
    See the `Measure <../../api/user-guide/programming-framework/supporting-elements.html#measure>`_
    section in the Programming Framework docs for more context.

    Measures should be created with the functions in :py:mod:`opendp.measures`
    or :py:mod:`opendp.context`, for a higher-level interface:

    >>> import opendp.prelude as dp
    >>> measure, distance = dp.loss_of(epsilon=1.0)
    >>> measure, distance
    (MaxDivergence, 1.0)

    '''
    _type_ = AnyMeasure

    @property
    def type(self):
        '''
        Type of measure
        '''
        from opendp.measures import measure_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measure_type(self))
    
    @property
    def distance_type(self) -> Union["RuntimeType", str]:
        '''
        Distance type of measure
        '''
        from opendp.measures import measure_distance_type
        from opendp.typing import RuntimeType
        return RuntimeType.parse(measure_distance_type(self))

    def __repr__(self):
        from opendp.measures import measure_debug
        return measure_debug(self)
    
    def __del__(self):
        try:
            from opendp.measures import _measure_free
            _measure_free(self)
        except (ImportError, TypeError): # pragma: no cover
            # an example error that this catches:
            #   ImportError: sys.meta_path is None, Python is likely shutting down
            pass

    def __eq__(self, other):
        from opendp.measures import _measure_equal

        if not isinstance(other, Measure):
            return False
        
        return _measure_equal(self, other)
    
    def __hash__(self) -> int:
        return hash(str(self))
    
    def __iter__(self):
        raise ValueError("Measure does not support iteration")



class ApproximateDivergence(Measure):
    '''``ApproximateDivergence`` is a privacy measure representing the divergence between two distributions 
    with respect to some inner privacy measure, except for some subset of possible outputs with probability mass no greater than delta.
    '''

    _type_ = AnyMeasure
    
    @property
    def inner_measure(self) -> Measure:
        from opendp.measures import _approximate_divergence_get_inner_measure
        return _approximate_divergence_get_inner_measure(self)
    

class PrivacyProfile(object):
    '''
    Given a profile function provided by the user,
    gives the epsilon corresponding to a given delta, and vice versa.

    :py:func:`new_privacy_profile <opendp.measures.new_privacy_profile>`
    should be used to create new instances.
    '''
    def __init__(self, curve):
        self.curve = curve

    def delta(self, epsilon):
        '''
        Returns the delta that corresponds to this epsilon.
        
        :param epsilon: Allowance for a multiplicative difference, or max divergence, in the distributions of releases on adjacent datasets
        '''
        from opendp._data import privacy_profile_delta
        return privacy_profile_delta(self.curve, epsilon)

    def epsilon(self, delta):
        '''
        Returns the epsilon that corresponds to this delta.
        
        :param delta: Allowance for an additive difference between the distributions of releases on adjacent datasets
        '''
        from opendp._data import privacy_profile_epsilon
        return privacy_profile_epsilon(self.curve, delta)
    

class _PartialConstructor(object):
    '''

    '''
    def __init__(self, constructor):
        self.constructor = constructor
        self.__opendp_dict__ = {}  # Not needed at runtime, but the definition prevents mypy errors.
    
    def __call__(self, input_domain: Domain, input_metric: Metric):
        return self.constructor(input_domain, input_metric)
    
    def __rshift__(self, other):
        return _PartialConstructor(lambda input_domain, input_metric: self(input_domain, input_metric) >> other) # pragma: no cover

    def __rrshift__(self, other):
        if isinstance(other, tuple):  
            domain, metric = other
            if isinstance(domain, Domain) and isinstance(metric, Metric):  
                return self(domain, metric)  
            
        raise TypeError(f"Cannot chain {type(self)} with {type(other)}")  # pragma: no cover


class UnknownTypeException(Exception):
    pass


class OpenDPException(Exception):
    """General exception for errors originating from the underlying OpenDP library.
    The variant attribute corresponds to `one of the following variants <https://github.com/opendp/opendp/blob/53ec58d01762ca5ceee08590d7e7b725bbdafcf6/rust/opendp/src/error.rs#L46-L87>`_ and can be matched on.
    Error variants may change in library updates.

    See `Rust ErrorVariant <https://docs.rs/opendp/latest/opendp/error/enum.ErrorVariant.html>`_ for values variant may take on.

    Run ``dp.enable_features('rust-stack-trace')`` to see wrapped Rust stack traces.
    """
    raw_traceback: Optional[str]

    def __init__(self, variant: str, message: Optional[str] = None, raw_traceback: Optional[str] = None):
        self.variant = variant
        self.message = message
        self.raw_traceback = raw_traceback

    def _raw_frames(self):
        import re
        return re.split(r"\s*[0-9]+: ", self.raw_traceback or "")
    
    def _frames(self):
        def _format_frame(frame):
            return "\n  ".join(line.strip() for line in frame.split("\n"))
        return [_format_frame(f) for f in self._raw_frames() if "opendp" in f]

    def _continued_stack_trace(self):
        # join and split by newlines because frames may be multi-line
        lines = "\n".join(self._frames()[::-1]).split('\n')
        return "Continued Rust stack trace:\n" + '\n'.join('    ' + line for line in lines)

    def __str__(self) -> str:
        '''
        >>> raw_traceback = """
        ... 0: top
        ... 1: opendp single line
        ... 2: opendp multi
        ...             line
        ... 3: bottom
        ... """
        >>> e = OpenDPException(variant='SomeVariant', message='my message', raw_traceback=raw_traceback)
        >>> dp.enable_features('rust-stack-trace')
        >>> print(e)
        Continued Rust stack trace:
            opendp multi
              line
            opendp single line
          SomeVariant("my message")
        >>> dp.disable_features('rust-stack-trace')
        >>> print(e)
        <BLANKLINE>
          SomeVariant("my message")
        '''
        response = ''
        if self.raw_traceback and 'rust-stack-trace' in GLOBAL_FEATURES:
            response += self._continued_stack_trace()
        response += '\n  ' + self.variant

        if self.message:
            response += f'("{self.message}")'
            
        return response


GLOBAL_FEATURES: set[str] = set()


def enable_features(*features: str) -> None:
    '''
    Allow the use of optional features. See :ref:`feature-listing` for details.
    '''
    GLOBAL_FEATURES.update(set(features))


def disable_features(*features: str) -> None:
    '''
    Disallow the use of optional features. See :ref:`feature-listing` for details.
    '''
    GLOBAL_FEATURES.difference_update(set(features))


def assert_features(*features: str) -> None:
    '''
    Check whether a given feature is enabled. See :ref:`feature-listing` for details.
    '''
    missing_features = [f for f in features if f not in GLOBAL_FEATURES]
    if missing_features:
        features_string = ', '.join(f'"{f}"' for f in features)
        raise OpenDPException(f"Attempted to use function that requires {features_string}, but not enabled. See https://github.com/opendp/opendp/discussions/304, then call enable_features({features_string})")


M = TypeVar("M", Transformation, Measurement)

def binary_search_chain(
        make_chain: Callable[[float], M],
        d_in: Any, d_out: Any,
        bounds: tuple[float, float] | None = None,
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


    :example:

    Find a laplace measurement with the smallest noise scale that is still (d_in, d_out)-close.

    >>> import opendp.prelude as dp
    >>> dp.enable_features("floating-point", "contrib")
    ...
    >>> # The majority of the chain only needs to be defined once.
    >>> pre = (
    ...     dp.space_of(list[float]) >>
    ...     dp.t.then_impute_constant(0.0) >>
    ...     dp.t.then_clamp(bounds=(0., 1.)) >>
    ...     dp.t.then_resize(size=10, constant=0.) >>
    ...     dp.t.then_mean()
    ... )
    ...
    >>> # Find a value in `bounds` that produces a (`d_in`, `d_out`)-chain nearest the decision boundary.
    >>> # The lambda function returns the complete computation chain when given a single numeric parameter.
    >>> chain = dp.binary_search_chain(
    ...     lambda s: pre >> dp.m.then_laplace(scale=s), 
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
        bounds: tuple[float, float] | None = None,
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
    ...     return dp.m.make_laplace(dp.atom_domain(T=float, nan=False), dp.absolute_distance(T=float), scale)
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
    ...        dp.m.then_laplace(necessary_scale)
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
        bounds: tuple[float, float] | None = ...,
        T: Type[float] | None = ...,
        return_sign: Literal[False] = False) -> float:
    ...


# when setting return sign to true as a keyword argument, return both
@overload
def binary_search(
        predicate: Callable[[float], bool],
        bounds: tuple[float, float] | None = ...,
        T: Type[float] | None = ...,
        *, # see https://stackoverflow.com/questions/66435480/overload-following-optional-argument
        return_sign: Literal[True]) -> tuple[float, int]:
    ...

# when setting return sign to true as a positional argument, return both
@overload
def binary_search(
        predicate: Callable[[float], bool],
        bounds: tuple[float, float] | None,
        T: Type[float] | None,
        return_sign: Literal[True]) -> tuple[float, int]:
    ...


def binary_search(
        predicate: Callable[[float], bool],
        bounds: tuple[float, float] | None = None,
        T: Type[float] | None = None,
        return_sign: bool = False) -> float | tuple[float, int]:
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

    >>> import opendp.prelude as dp
    >>> dp.binary_search(lambda x: x >= 5.)
    5.0
    >>> dp.binary_search(lambda x: x <= 5.)
    5.0
    >>> dp.binary_search(lambda x: x > 5, T=int)
    6
    >>> dp.binary_search(lambda x: x < 5, T=int)
    4

    Find epsilon usage of the gaussian(scale=1.) mechanism applied on a dp mean.
    Assume neighboring datasets differ by up to three additions/removals, and fix delta to 1e-8.

    >>> # build a histogram that emits float counts
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
        raise ValueError("unable to infer bounds")  # pragma: no cover

    if len(set(map(type, bounds))) != 1:
        raise TypeError("bounds must share the same type")  # pragma: no cover
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
        raise TypeError("bounds must be either float or int")  # pragma: no cover

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


_EXPONENTIAL_SEARCH_BANDS: dict[Type, list[float]] = {
    # Searching bands of [(k - 1) * 2^16, k * 2^16].
    # Integers have linear space between bands.
    # Additionally include 1 because zero is prone to error.
    int: [0, 1, *(2 ** 16 * k for k in range(1, 9))],

    # Searching bands of [2^((k - 1)^2), 2^(k^2)].
    # Exponent has ten bits (2^1024 overflows) so k must be in [0, 32).
    # Unlikely to need numbers greater than 2**64, and to avoid overflow from shifted centers,
    #    only check k in [0, 8). 
    # Set your own bounds if this is not sufficient.
    float: [0.0, 0.5, *(2. ** k ** 2 for k in range(1024 // 32 // 4))]
}

def exponential_bounds_search(
    predicate: Callable[[float], bool], T: Optional[Union[Type[float], Type[int]]]
) -> Optional[tuple[float, float]]:
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
    T = T or _infer_type(predicate)
                
    # core search functionality
    def signed_band_search(center, at_center, sign):
        """Identify which band (of eight) the decision boundary lies in.

        :param center: Start here
        :param at_center: How the predicate evaluates at `center`. Search terminates when predicate changes
        :param sign: Search in this direction
        """

        if T not in _EXPONENTIAL_SEARCH_BANDS:
            raise TypeError(f"unknown type {T}. Must be one of int, float")  # pragma: no cover
        bands = [center + sign * c for c in _EXPONENTIAL_SEARCH_BANDS[T]]

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
    def _exception_predicate(v):
        try:
            predicate(v)
            return True
        except Exception:
            return False
    
    exception_bounds = exponential_bounds_search(_exception_predicate, T=T)
    if exception_bounds is None:
        try:
            predicate(center)
        except Exception as e:
            # enrich the error message if in Python 3.11+.
            if hasattr(e, "add_note"):
                e.add_note(f"Predicate in binary search always raises an exception. This exception is raised when the predicate is evaluated at {center}.")
            raise
    

    center, sign = binary_search(_exception_predicate, bounds=exception_bounds, T=T, return_sign=True)
    at_center = predicate(center)
    return signed_band_search(center, at_center, sign)


def _infer_type(predicate: Callable[[float], bool]) -> Union[Type[float], Type[int]]:
    def _is_type_error(e):
        is_match_error = isinstance(e, OpenDPException) and "No match for concrete type" in (e.message or "")
        is_type_error = isinstance(e, TypeError)
        return is_match_error or is_type_error

    try:
        predicate(0.)
        # predicate succeeded with a float, assume it wants floats
        return float
    except Exception as e:
        if not _is_type_error(e):
            # it didn't reject the type, but did fail with a different error
            return float
    
    # Try again with the more forgiving type, an int.
    try:
        predicate(0)
        # predicate is happy with an int, assume it wants ints
        return int
    except Exception as e:
        if not _is_type_error(e):
            # it didn't reject the type, but did fail with a different error
            return int

        # enrich the error message if in Python 3.11+.
        if hasattr(e, "add_note"):
            e.add_note("Unable to infer type `T`; pass the type `T` or bounds. This exception is raised when the predicate is evaluated at 0")
        raise


_TUPLE_FLAG = '__tuple__'
_EXPR_FLAG = '__expr__'
_LAZY_FLAG = '__lazyframe__'
_MARGIN_FLAG = '__margin__'
_FUNCTION_FLAG = '__function__'
_MODULE_FLAG = '__module__'
_KWARGS_FLAG = '__kwargs__'

def _bytes_to_b64_str(serialized_polars):
    from base64 import b64encode
    b64_bytes = b64encode(serialized_polars)
    return b64_bytes.decode()

def _b64_str_to_bytes(b64_str):
    from base64 import b64decode
    from io import BytesIO
    b64_bytes = b64_str.encode()
    serialized_polars = b64decode(b64_bytes)
    return BytesIO(serialized_polars)

class _Encoder(json.JSONEncoder):
    def default(self, obj):
        # Basic types:
        if isinstance(obj, (str, int, float, bool, type(None))):
            return obj
        if isinstance(obj, list):
            return [self.default(value) for value in obj]
        if isinstance(obj, tuple):
            return {_TUPLE_FLAG: [self.default(value) for value in obj]}
        if isinstance(obj, dict):
            return {
                # Dict keys must be hashable,
                # and with the JSON serialization we can't use tuple keys either,
                # so no need to recurse on the key.
                k: self.default(v)
                for k, v in obj.items()
            }
        
        from opendp.extras.polars import Margin
        if isinstance(obj, Margin):
            return {_MARGIN_FLAG: self.default(asdict(obj))}

        # OpenDP specific:
        if hasattr(obj, '__opendp_dict__'):
            return self.default({**obj.__opendp_dict__, '__version__': __version__})

        stateful_error_msg = (
            f"OpenDP JSON Encoder does not handle instances of {type(obj)}: "
            f"It may have state which is not set by the constructor. Error on: {obj}"
        )

        pl = import_optional_dependency('polars', raise_error=False)
        if pl is not None:
            from opendp.extras.polars import LazyFrameQuery
            if isinstance(obj, LazyFrameQuery):
                # Error out early, instead of falling through to pl.LazyFrame,
                # which *could* be serialized!
                raise Exception(stateful_error_msg)
            if isinstance(obj, pl.Expr):
                return {_EXPR_FLAG: _bytes_to_b64_str(obj.meta.serialize()), '__version__': __version__}
            if isinstance(obj, pl.LazyFrame):
                return {_LAZY_FLAG: _bytes_to_b64_str(obj.serialize()), '__version__': __version__}

        # Exceptions:
        from opendp.context import Context
        if isinstance(obj, (Context, Queryable,)):
            raise Exception(stateful_error_msg)

        raise Exception(f'OpenDP JSON Encoder does not handle {obj}')

def _check_version(dp_dict):
    from warnings import warn
    serialized_version = dp_dict['__version__']
    if serialized_version != __version__:
        warn(
            f'OpenDP version in serialized object ({serialized_version}) '
            f'!= this version ({__version__})')

def _deserialization_hook(dp_dict):
    if _FUNCTION_FLAG in dp_dict:
        _check_version(dp_dict)
        module = importlib.import_module(f"opendp.{dp_dict[_MODULE_FLAG]}")
        func = getattr(module, dp_dict[_FUNCTION_FLAG])
        return func(**dp_dict.get(_KWARGS_FLAG, {}))
    if _TUPLE_FLAG in dp_dict:
        return tuple(dp_dict[_TUPLE_FLAG])
    pl = import_optional_dependency('polars', raise_error=False)
    if pl is not None:
        if _EXPR_FLAG in dp_dict:
            _check_version(dp_dict)
            return pl.Expr.deserialize(_b64_str_to_bytes(dp_dict[_EXPR_FLAG]))
        if _LAZY_FLAG in dp_dict:
            _check_version(dp_dict)
            return pl.LazyFrame.deserialize(_b64_str_to_bytes(dp_dict[_LAZY_FLAG]))
        if _MARGIN_FLAG in dp_dict:
            from opendp.extras.polars import Margin

            by = [deserialize(v) for v in dp_dict[_MARGIN_FLAG].get('by', [])]
            return Margin(**{**dp_dict[_MARGIN_FLAG], "by": by})
    return dp_dict

def serialize(dp_obj):
    # The usual pattern would be
    # 
    #   json.dumps(dp_obj, cls=_Encoder)
    # 
    # but that only calls default() for objects which can't be handled otherwise.
    # In particular, it makes top-level tuples into lists,
    # even when special handling is specified in default().
    return json.dumps(_Encoder().default(dp_obj))

def deserialize(dp_json):
    return json.loads(dp_json, object_hook=_deserialization_hook)


_EXPECTED_POLARS_VERSION = '1.32.0' # Keep in sync with setup.cfg.


__version__ = get_opendp_version()
