# Auto-generated. Do not edit!
'''
The ``measures`` module provides functions that measure the distance between probability distributions.
For more context, see :ref:`measures in the User Guide <measures-user-guide>`.

For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp
'''
from deprecated.sphinx import deprecated # noqa: F401 (Not every file actually has deprecated functions.)

from opendp._convert import *
from opendp._lib import *
from opendp.mod import *
from opendp.typing import *
from opendp.typing import _substitute # noqa: F401

__all__ = [
    "_approximate_divergence_get_inner_measure",
    "_measure_equal",
    "_measure_free",
    "approximate",
    "fixed_smoothed_max_divergence",
    "max_divergence",
    "measure_debug",
    "measure_distance_type",
    "measure_type",
    "new_privacy_profile",
    "renyi_divergence",
    "smoothed_max_divergence",
    "user_divergence",
    "zero_concentrated_divergence"
]


def _approximate_divergence_get_inner_measure(
    privacy_measure: Measure
) -> Measure:
    r"""Retrieve the inner privacy measure of an approximate privacy measure.

    .. end-markdown

    :param privacy_measure: The privacy measure to inspect
    :type privacy_measure: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_privacy_measure = py_to_c(privacy_measure, c_type=Measure, type_name="AnyMeasure")

    # Call library function.
    lib_function = lib.opendp_measures___approximate_divergence_get_inner_measure
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_privacy_measure), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': '_approximate_divergence_get_inner_measure',
            '__module__': 'measures',
            '__kwargs__': {
                'privacy_measure': privacy_measure
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _measure_equal(
    left: Measure,
    right: Measure
) -> bool:
    r"""Check whether two measures are equal.

    .. end-markdown

    :param left: Measure to compare.
    :type left: Measure
    :param right: Measure to compare.
    :type right: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_left = py_to_c(left, c_type=Measure, type_name="AnyMeasure")
    c_right = py_to_c(right, c_type=Measure, type_name="AnyMeasure")

    # Call library function.
    lib_function = lib.opendp_measures___measure_equal
    lib_function.argtypes = [Measure, Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_left, c_right), BoolPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': '_measure_equal',
            '__module__': 'measures',
            '__kwargs__': {
                'left': left, 'right': right
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def _measure_free(
    this
):
    r"""Internal function. Free the memory associated with `this`.

    .. end-markdown

    :param this: 
    :type this: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = this

    # Call library function.
    lib_function = lib.opendp_measures___measure_free
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_void_p))
    try:
        output.__opendp_dict__ = {
            '__function__': '_measure_free',
            '__module__': 'measures',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def approximate(
    measure: Measure
) -> ApproximateDivergence:
    r"""Privacy measure used to define $\delta$-approximate PM-differential privacy.

    In the following definition, $d$ corresponds to privacy parameters $(d', \delta)$
    when also quantified over all adjacent datasets
    ($d'$ is the privacy parameter corresponding to privacy measure PM).
    That is, $(d', \delta)$ is no smaller than $d$ (by product ordering),
    over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
    $M(\cdot)$ is a measurement (commonly known as a mechanism).
    The measurement's input metric defines the notion of adjacency,
    and the measurement's input domain defines the set of possible datasets.

    [approximate in Rust documentation.](https://docs.rs/opendp/0.14.1-nightly.20260130.1/opendp/measures/struct.Approximate.html)

    **Proof Definition:**

    For any two distributions $Y, Y'$ and 2-tuple $d = (d', \delta)$,
    where $d'$ is the distance with respect to privacy measure PM,
    $Y, Y'$ are $d$-close under the approximate PM measure whenever,
    for any choice of $\delta \in [0, 1]$,
    there exist events $E$ (depending on $Y$) and $E'$ (depending on $Y'$)
    such that $\Pr[E] \ge 1 - \delta$, $\Pr[E'] \ge 1 - \delta$, and

    $D_{\mathrm{PM}}^\delta(Y|_E, Y'|_{E'}) = D_{\mathrm{PM}}(Y|_E, Y'|_{E'})$

    where $Y|_E$ denotes the distribution of $Y$ conditioned on the event $E$.

    Note that this $\delta$ is not privacy parameter $\delta$ until quantified over all adjacent datasets,
    as is done in the definition of a measurement.

    .. end-markdown

    :param measure: inner privacy measure
    :type measure: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_measure = py_to_c(measure, c_type=Measure, type_name=None)

    # Call library function.
    lib_function = lib.opendp_measures__approximate
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_measure), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'approximate',
            '__module__': 'measures',
            '__kwargs__': {
                'measure': measure
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def fixed_smoothed_max_divergence(

) -> Measure:
    r"""Privacy measure used to define $(\epsilon, \delta)$-approximate differential privacy.

    In the following definition, $d$ corresponds to $(\epsilon, \delta)$ when also quantified over all adjacent datasets.
    That is, $(\epsilon, \delta)$ is no smaller than $d$ (by product ordering),
    over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
    $M(\cdot)$ is a measurement (commonly known as a mechanism).
    The measurement's input metric defines the notion of adjacency,
    and the measurement's input domain defines the set of possible datasets.

    **Proof Definition:**

    For any two distributions $Y, Y'$ and any 2-tuple $d$ of non-negative numbers $\epsilon$ and $\delta$,
    $Y, Y'$ are $d$-close under the fixed smoothed max divergence measure whenever

    $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.

    Note that this $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
    as is done in the definition of a measurement.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_measures__fixed_smoothed_max_divergence
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'fixed_smoothed_max_divergence',
            '__module__': 'measures',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def max_divergence(

) -> Measure:
    r"""Privacy measure used to define $\epsilon$-pure differential privacy.

    In the following proof definition, $d$ corresponds to $\epsilon$ when also quantified over all adjacent datasets.
    That is, $\epsilon$ is the greatest possible $d$
    over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
    $M(\cdot)$ is a measurement (commonly known as a mechanism).
    The measurement's input metric defines the notion of adjacency,
    and the measurement's input domain defines the set of possible datasets.

    **Proof Definition:**

    For any two distributions $Y, Y'$ and any non-negative $d$,
    $Y, Y'$ are $d$-close under the max divergence measure whenever

    $D_\infty(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S]}{\Pr[Y' \in S]} \Big] \leq d$.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_measures__max_divergence
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'max_divergence',
            '__module__': 'measures',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def measure_debug(
    this: Measure
) -> str:
    r"""Debug a `measure`.

    .. end-markdown

    :param this: The measure to debug (stringify).
    :type this: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)

    # Call library function.
    lib_function = lib.opendp_measures__measure_debug
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    try:
        output.__opendp_dict__ = {
            '__function__': 'measure_debug',
            '__module__': 'measures',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def measure_distance_type(
    this: Measure
) -> str:
    r"""Get the distance type of a `measure`.

    .. end-markdown

    :param this: The measure to retrieve the distance type from.
    :type this: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)

    # Call library function.
    lib_function = lib.opendp_measures__measure_distance_type
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    try:
        output.__opendp_dict__ = {
            '__function__': 'measure_distance_type',
            '__module__': 'measures',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def measure_type(
    this: Measure
) -> str:
    r"""Get the type of a `measure`.

    .. end-markdown

    :param this: The measure to retrieve the type from.
    :type this: Measure
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # Convert arguments to c types.
    c_this = py_to_c(this, c_type=Measure, type_name=None)

    # Call library function.
    lib_function = lib.opendp_measures__measure_type
    lib_function.argtypes = [Measure]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_this), ctypes.c_char_p))
    try:
        output.__opendp_dict__ = {
            '__function__': 'measure_type',
            '__module__': 'measures',
            '__kwargs__': {
                'this': this
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def new_privacy_profile(
    curve
):
    r"""Construct a PrivacyProfile from a user-defined callback.


    Required features: `contrib`, `honest-but-curious`

    **Why honest-but-curious?:**

    The privacy profile should implement a well-defined $\delta(\epsilon)$ curve:

    * monotonically decreasing
    * rejects epsilon values that are less than zero or nan
    * returns delta values only within [0, 1]

    .. end-markdown

    :param curve: A privacy curve mapping epsilon to delta
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library

    :example:

        >>> dp.enable_features("contrib", "honest-but-curious")
        >>> profile = dp.new_privacy_profile(lambda eps: 1.0 if eps < 0.5 else 1e-8)
        ...
        >>> # epsilon is not enough, so delta saturates to one
        >>> profile.delta(epsilon=0.499)
        1.0
        >>> # invert it, find the suitable epsilon at this delta
        >>> profile.epsilon(delta=1e-8)
        0.5
        >>> # insufficient delta results in infinite epsilon
        >>> profile.epsilon(delta=1e-9)
        inf


    """
    assert_features("contrib", "honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_curve = py_to_c(curve, c_type=CallbackFnPtr, type_name="f64")

    # Call library function.
    lib_function = lib.opendp_measures__new_privacy_profile
    lib_function.argtypes = [CallbackFnPtr]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_curve), AnyObjectPtr))
    try:
        output.__opendp_dict__ = {
            '__function__': 'new_privacy_profile',
            '__module__': 'measures',
            '__kwargs__': {
                'curve': curve
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def renyi_divergence(

) -> Measure:
    r"""Privacy measure used to define $\epsilon(\alpha)$-Rényi differential privacy.

    In the following proof definition, $d$ corresponds to an RDP curve when also quantified over all adjacent datasets.
    That is, an RDP curve $\epsilon(\alpha)$ is no smaller than $d(\alpha)$ for any possible choices of $\alpha$,
    and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
    $M(\cdot)$ is a measurement (commonly known as a mechanism).
    The measurement's input metric defines the notion of adjacency,
    and the measurement's input domain defines the set of possible datasets.

    **Proof Definition:**

    For any two distributions $Y, Y'$ and any curve $d$,
    $Y, Y'$ are $d$-close under the Rényi divergence measure if,
    for any given $\alpha \in (1, \infty)$,

    $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d(\alpha)$

    Note that this $\epsilon$ and $\alpha$ are not privacy parameters $\epsilon$ and $\alpha$ until quantified over all adjacent datasets,
    as is done in the definition of a measurement.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_measures__renyi_divergence
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'renyi_divergence',
            '__module__': 'measures',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def smoothed_max_divergence(

) -> Measure:
    r"""Privacy measure used to define $\epsilon(\delta)$-approximate differential privacy.

    In the following proof definition, $d$ corresponds to a privacy profile when also quantified over all adjacent datasets.
    That is, a privacy profile $\epsilon(\delta)$ is no smaller than $d(\delta)$ for all possible choices of $\delta$,
    and over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
    $M(\cdot)$ is a measurement (commonly known as a mechanism).
    The measurement's input metric defines the notion of adjacency,
    and the measurement's input domain defines the set of possible datasets.

    The distance $d$ is of type PrivacyProfile, so it can be invoked with an $\epsilon$
    to retrieve the corresponding $\delta$.

    **Proof Definition:**

    For any two distributions $Y, Y'$ and any curve $d(\cdot)$,
    $Y, Y'$ are $d$-close under the smoothed max divergence measure whenever,
    for any choice of non-negative $\epsilon$, and $\delta = d(\epsilon)$,

    $D_\infty^\delta(Y, Y') = \max_{S \subseteq \textrm{Supp}(Y)} \Big[\ln \dfrac{\Pr[Y \in S] + \delta}{\Pr[Y' \in S]} \Big] \leq \epsilon$.

    Note that $\epsilon$ and $\delta$ are not privacy parameters $\epsilon$ and $\delta$ until quantified over all adjacent datasets,
    as is done in the definition of a measurement.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_measures__smoothed_max_divergence
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'smoothed_max_divergence',
            '__module__': 'measures',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def user_divergence(
    descriptor: str
) -> Measure:
    r"""Privacy measure with meaning defined by an OpenDP Library user (you).

    Any two instances of UserDivergence are equal if their string descriptors are equal.


    Required features: `honest-but-curious`

    **Why honest-but-curious?:**

    The essential requirement of a privacy measure is that it is closed under postprocessing.
    Your privacy measure `D` must satisfy that, for any pure function `f` and any two distributions `Y, Y'`, then $D(Y, Y') \ge D(f(Y), f(Y'))$.

    Beyond this, you should also consider whether your privacy measure can be used to provide meaningful privacy guarantees to your privacy units.

    .. end-markdown

    :param descriptor: A string description of the privacy measure.
    :type descriptor: str
    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    assert_features("honest-but-curious")

    # No type arguments to standardize.
    # Convert arguments to c types.
    c_descriptor = py_to_c(descriptor, c_type=ctypes.c_char_p, type_name="String")

    # Call library function.
    lib_function = lib.opendp_measures__user_divergence
    lib_function.argtypes = [ctypes.c_char_p]
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(c_descriptor), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'user_divergence',
            '__module__': 'measures',
            '__kwargs__': {
                'descriptor': descriptor
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output


def zero_concentrated_divergence(

) -> Measure:
    r"""Privacy measure used to define $\rho$-zero concentrated differential privacy.

    In the following proof definition, $d$ corresponds to $\rho$ when also quantified over all adjacent datasets.
    That is, $\rho$ is the greatest possible $d$
    over all pairs of adjacent datasets $x, x'$ where $Y \sim M(x)$, $Y' \sim M(x')$.
    $M(\cdot)$ is a measurement (commonly known as a mechanism).
    The measurement's input metric defines the notion of adjacency,
    and the measurement's input domain defines the set of possible datasets.

    **Proof Definition:**

    For any two distributions $Y, Y'$ and any non-negative $d$,
    $Y, Y'$ are $d$-close under the zero-concentrated divergence measure if,
    for every possible choice of $\alpha \in (1, \infty)$,

    $D_\alpha(Y, Y') = \frac{1}{1 - \alpha} \mathbb{E}_{x \sim Y'} \Big[\ln \left( \dfrac{\Pr[Y = x]}{\Pr[Y' = x]} \right)^\alpha \Big] \leq d \cdot \alpha$.

    .. end-markdown


    :raises TypeError: if an argument's type differs from the expected type
    :raises UnknownTypeException: if a type argument fails to parse
    :raises OpenDPException: packaged error from the core OpenDP library
    """
    # No type arguments to standardize.
    # No arguments to convert to c types.
    # Call library function.
    lib_function = lib.opendp_measures__zero_concentrated_divergence
    lib_function.argtypes = []
    lib_function.restype = FfiResult

    output = c_to_py(unwrap(lib_function(), Measure))
    try:
        output.__opendp_dict__ = {
            '__function__': 'zero_concentrated_divergence',
            '__module__': 'measures',
            '__kwargs__': {
                
            },
        }
    except AttributeError:  # pragma: no cover
        pass
    return output
