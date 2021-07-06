import ctypes
from typing import Union

from opendp.v1._lib import AnyMeasurement, AnyTransformation


class Measurement(ctypes.POINTER(AnyMeasurement)):
    """A differentially private unit of computation.
    A measurement contains a function and a privacy relation.
    The function releases a differentially-private release.
    The privacy relation maps from an input metric to an output measure.

    :example:

    >>> from opendp.v1.mod import Measurement
    >>>
    >>> # create an instance of Measurement using a constructor from the meas module
    >>> from opendp.v1.meas import make_base_geometric
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
    >>> from opendp.v1.trans import make_count
    >>> from opendp.v1.typing import HammingDistance
    >>> chained = (
    >>>     make_count(MI=HammingDistance, TI=int) >>
    >>>     base_geometric
    >>> )
    >>>
    >>> # the resulting measurement has the same features
    >>> chained([1, 2, 3])  # -> 4
    >>> # check the chained measurement's relation at
    >>> #     (1, 0.5): (HammingDistance, MaxDivergence)
    >>> assert chained.check(1, 0.5)
    """
    _type_ = AnyMeasurement

    def __call__(self, arg):
        from opendp.v1.core import measurement_invoke
        return measurement_invoke(self, arg)

    def invoke(self, arg):
        """Create a differentially-private release with `arg`.
        
        :param arg: Input to the measurement.
        :return: differentially-private release
        :raises OpenDPException: packaged error from the core OpenDP library
        """
        from opendp.v1.core import measurement_invoke
        return measurement_invoke(self, arg)

    def check(self, d_in, d_out, *, debug=False) -> bool:
        """Check if the measurement satisfies the privacy relation at `d_in`, `d_out`.
        
        :param d_in: Distance in terms of the input metric.
        :param d_out: Distance in terms of the output measure.
        :param debug: Enable to raise Exceptions to help identify why the privacy relation failed.
        :return: If True, a release is differentially private at `d_in`, `d_out`.
        :rtype: bool
        """
        from opendp.v1.core import measurement_check

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
        from opendp.v1.core import measurement_input_distance_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(measurement_input_distance_type(self))

    @property
    def output_distance_type(self):
        """Retrieve the distance type of the output measure.
        This is the type that the budget is expressed in.
        
        :return: distance type
        """
        from opendp.v1.typing import RuntimeType
        from opendp.v1.core import measurement_output_distance_type
        return RuntimeType.parse(measurement_output_distance_type(self))

    @property
    def input_carrier_type(self):
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.
        
        :return: carrier type
        """
        from opendp.v1.core import measurement_input_carrier_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(measurement_input_carrier_type(self))

    def __del__(self):
        from opendp.v1.core import _measurement_free
        _measurement_free(self)


class Transformation(ctypes.POINTER(AnyTransformation)):
    """A non-differentially private unit of computation.
    A transformation contains a function and a stability relation.
    The function maps from an input domain to an output domain.
    The stability relation maps from an input metric to an output metric.

    :example:

    >>> from opendp.v1.mod import Transformation
    >>>
    >>> # create an instance of Transformation using a constructor from the trans module
    >>> from opendp.v1.trans import make_count
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
    >>> from opendp.v1.trans import make_split_lines, make_cast
    >>> from opendp.v1.typing import SymmetricDistance
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
        from opendp.v1.core import transformation_invoke
        return transformation_invoke(self, arg)

    def __call__(self, arg):
        from opendp.v1.core import transformation_invoke
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
        from opendp.v1.core import transformation_check

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
            from opendp.v1.core import make_chain_mt
            return make_chain_mt(other, self)

        if isinstance(other, Transformation):
            from opendp.v1.core import make_chain_tt
            return make_chain_tt(other, self)

        raise ValueError(f"rshift expected a measurement or transformation, got {other}")

    @property
    def input_distance_type(self):
        """Retrieve the distance type of the input metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.

        :return: distance type
        """
        from opendp.v1.core import transformation_input_distance_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(transformation_input_distance_type(self))

    @property
    def output_distance_type(self):
        """Retrieve the distance type of the output metric.
        This may be any integral type for dataset metrics, or any numeric type for sensitivity metrics.

        :return: distance type
        """
        from opendp.v1.core import transformation_output_distance_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(transformation_output_distance_type(self))
    
    @property
    def input_carrier_type(self):
        """Retrieve the carrier type of the input domain.
        Any member of the input domain is a member of the carrier type.

        :return: carrier type
        """
        from opendp.v1.core import transformation_input_carrier_type
        from opendp.v1.typing import RuntimeType
        return RuntimeType.parse(transformation_input_carrier_type(self))

    def __del__(self):
        from opendp.v1.core import _transformation_free
        _transformation_free(self)


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
