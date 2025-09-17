"""
The ``context`` module provides :py:class:`opendp.context.Context` and supporting utilities.

For more context, see :ref:`context in the User Guide <context-user-guide>`.

For convenience, all the members of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: pycon

    >>> import opendp.prelude as dp
"""

import logging
from typing import Any, Callable, Optional, Sequence, Type, Union, MutableMapping
import importlib
from inspect import signature
from functools import partial
from opendp.combinators import (
    make_fix_delta,
    make_approximate,
    make_fully_adaptive_composition,
    make_privacy_filter,
    make_pureDP_to_zCDP,
    make_adaptive_composition,
    make_zCDP_to_approxDP,
)
from opendp.domains import atom_domain, vector_domain, with_margin
from opendp.extras._utilities import supports_partial, to_then
from opendp.measurements import make_laplace, make_gaussian
from opendp.measures import (
    approximate,
    max_divergence,
    zero_concentrated_divergence,
)
from opendp.metrics import (
    absolute_distance,
    change_one_distance,
    change_one_id_distance,
    discrete_distance,
    hamming_distance,
    insert_delete_distance,
    l1_distance,
    l2_distance,
    frame_distance,
    symmetric_distance,
    symmetric_id_distance,
)
from opendp.mod import (
    ApproximateDivergence,
    Domain,
    Measurement,
    Metric,
    _PartialConstructor,
    Odometer,
    OdometerQueryable,
    Queryable,
    Transformation,
    Measure,
    binary_search,
    binary_search_param,
)
from opendp.typing import RuntimeType
from opendp._lib import indent, import_optional_dependency
from opendp.extras.polars import Bound, LazyFrameQuery, Margin
from dataclasses import replace

__all__ = [
    "space_of",
    "domain_of",
    "metric_of",
    "loss_of",
    "unit_of",
    "Context",
    "Query",
    "Chain",
    "PartialChain",
    "register",
]


logger = logging.getLogger(__name__)


# a dictionary of "constructor name" -> (constructor_function, is_partial)
# "constructor name" is the name of the constructor without the "make_" prefix
# constructor_function is the partial version if is_partial is True
constructors = {}
for module_name in ["transformations", "measurements"]:
    module = importlib.import_module(f"opendp.{module_name}")
    for name in module.__all__:
        if not name.startswith("make_"):
            continue
        partial_name = "then_" + name[5:]
        make_func = getattr(module, name)

        is_partial = partial_name in module.__all__
        constructor = getattr(module, partial_name if is_partial else name)

        constructors[name[5:]] = constructor, is_partial


def register(
    constructor: Callable[..., Union[Transformation, Measurement]],
    name: Optional[str] = None,
):
    """Register a constructor function to be used in the Context API.

    If the constructor supports partial application (first two arguments are ``input_domain`` and ``input_metric``),
    then the input domain and input metric are omitted when called via the Context API.

    Constructor requirements:

    * The constructor must return a transformation or measurement.
    * If name is None, the constructor's name must start with ``make_``.

    :param constructor: The constructor function to register.
    :param name: The name to register the constructor under in the Context API. If None, the name will be derived from the constructor's name.
    """
    if name is None:
        if not constructor.__name__.startswith("make_"):
            raise ValueError(
                f"constructor.__name__ must start with 'make_', found {constructor.__name__}"
            )
        name = constructor.__name__[5:]

    if name in constructors:
        raise ValueError(
            f"'{name}' is already registered in the Context API. Please choose a different name."
        )

    if supports_partial(constructor):
        constructors[name] = to_then(constructor), True
    else:
        constructors[name] = constructor, False


def space_of(T, M=None, infer: bool = False) -> tuple[Domain, Metric]:
    """A shorthand for building a metric space.

    A metric space consists of a domain and a metric.

    >>> import opendp.prelude as dp
    ...
    >>> dp.space_of(list[int])
    (VectorDomain(AtomDomain(T=i32)), SymmetricDistance())
    >>> # the verbose form allows greater control:
    >>> (dp.vector_domain(dp.atom_domain(T=dp.i32)), dp.symmetric_distance())
    (VectorDomain(AtomDomain(T=i32)), SymmetricDistance())

    :param T: carrier type (the type of members in the domain)
    :param M: metric type
    :param infer: if True, ``T`` is an example of the sensitive dataset. Passing sensitive data may result in a privacy violation.
    """
    import opendp.typing as ty

    domain = domain_of(T, infer=infer)
    D = domain.type

    # choose a metric type if not set
    if M is None:
        if D.origin == "VectorDomain":  # type: ignore[union-attr]
            M = ty.SymmetricDistance
        elif D.origin == "AtomDomain" and ty.get_atom(D) in ty._NUMERIC_TYPES:  # type: ignore[union-attr]
            M = ty.AbsoluteDistance
        else:
            raise TypeError(
                f"no default metric for domain {D}. Please set `M`"
            )  # pragma: no cover

    # choose a distance type if not set
    if isinstance(M, ty.RuntimeType) and not M.args:
        M = M[ty.get_atom(D)]  # type: ignore[index]

    return domain, metric_of(M)


def domain_of(T, infer: bool = False) -> Domain:
    """Constructs an instance of a domain from carrier type ``T``, or from an example.

    Accepts a limited set of Python type expressions:

    >>> import opendp.prelude as dp
    >>> dp.domain_of(list[int])
    VectorDomain(AtomDomain(T=i32))

    As well as strings representing types in the underlying Rust syntax:

    >>> dp.domain_of('Vec<int>')
    VectorDomain(AtomDomain(T=i32))

    Dictionaries, optional types, and a range of primitive types are supported:

    >>> dp.domain_of(dict[str, int])
    MapDomain { key_domain: AtomDomain(T=String), value_domain: AtomDomain(T=i32) }

    .. TODO: Support python syntax for Option: https://github.com/opendp/opendp/issues/1389

    >>> dp.domain_of('Option<int>')  # Python's `Optional` is not supported.
    OptionDomain(AtomDomain(T=i32))
    >>> dp.domain_of(dp.i32)
    AtomDomain(T=i32)

    More complex types are not supported:

    >>> dp.domain_of(list[list[int]]) # doctest: +IGNORE_EXCEPTION_DETAIL
    Traceback (most recent call last):
    ...
    opendp.mod.OpenDPException:
      FFI("Inner domain of VectorDomain must be AtomDomain or ExtrinsicDomain (created via user_domain)")

    Alternatively, an example of the data can be provided, but note that passing sensitive data may result in a privacy violation:

    >>> dp.domain_of([1, 2, 3], infer=True)
    VectorDomain(AtomDomain(T=i32))

    :param T: carrier type
    :param infer: if True, ``T`` is an example of the sensitive dataset. Passing sensitive data may result in a privacy violation.
    """
    import opendp.typing as ty
    from opendp.domains import vector_domain, atom_domain, option_domain, map_domain

    if infer:
        pl = import_optional_dependency("polars", raise_error=False)
        if pl is not None and isinstance(T, pl.LazyFrame):
            from opendp.extras.polars import _lazyframe_domain_from_schema

            return _lazyframe_domain_from_schema(T.collect_schema())

    # normalize to a type descriptor
    if infer:
        T = ty.RuntimeType.infer(T)
    else:
        T = ty.RuntimeType.parse(T)

    # construct the domain
    if isinstance(T, ty.RuntimeType):
        if T.origin == "Vec":
            return vector_domain(domain_of(T.args[0]))
        if T.origin == "HashMap":
            return map_domain(domain_of(T.args[0]), domain_of(T.args[1]))
        if T.origin == "Option":
            return option_domain(domain_of(T.args[0]))

    if T in ty._PRIMITIVE_TYPES:
        return atom_domain(T=T)

    raise TypeError(f"unrecognized carrier type: {T}")  # pragma: no cover


def metric_of(M) -> Metric:
    """Constructs an instance of a metric from metric type ``M``.

    :param M: Metric type
    """
    import opendp.typing as ty
    import opendp.metrics as metrics

    if isinstance(M, Metric):
        return M
    M = ty.RuntimeType.parse(M)

    if isinstance(M, ty.RuntimeType):
        if M.origin == "AbsoluteDistance":
            return metrics.absolute_distance(T=M.args[0])
        if M.origin == "L1Distance":
            return metrics.l1_distance(T=M.args[0])
        if M.origin == "L2Distance":
            return metrics.l2_distance(T=M.args[0])

    if M == ty.HammingDistance:
        return metrics.hamming_distance()
    if M == ty.SymmetricDistance:
        return metrics.symmetric_distance()
    if M == ty.InsertDeleteDistance:
        return metrics.insert_delete_distance()
    if M == ty.ChangeOneDistance:
        return metrics.change_one_distance()
    if M == ty.DiscreteDistance:
        return metrics.discrete_distance()

    raise TypeError(f"unrecognized metric: {M}")  # pragma: no cover


def loss_of(
    epsilon: Optional[float] = None,
    delta: Optional[float] = None,
    rho: Optional[float] = None,
) -> tuple[Measure, Union[float, tuple[float, float]]]:
    """Constructs a privacy loss, consisting of a privacy measure and a privacy loss parameter.

    >>> import opendp.prelude as dp
    >>> dp.loss_of(epsilon=1.0)
    (MaxDivergence, 1.0)
    >>> dp.loss_of(epsilon=1.0, delta=1e-9)
    (Approximate(MaxDivergence), (1.0, 1e-09))
    >>> dp.loss_of(rho=1.0)
    (ZeroConcentratedDivergence, 1.0)

    :param epsilon: Parameter for pure ε-DP.
    :param delta: Parameter for δ-approximate DP.
    :param rho: Parameter for zero-concentrated ρ-DP.

    """

    def _range_warning(name, value, info_level, warn_level):
        if value > warn_level:
            if info_level == warn_level:
                logger.warning(f"{name} should be less than or equal to {warn_level}")
            else:
                logger.warning(
                    f"{name} should be less than or equal to {warn_level}, and is typically less than or equal to {info_level}"
                )
        elif value > info_level:
            logger.info(f"{name} is typically less than or equal to {info_level}")

    if [rho is None, epsilon is None].count(True) != 1:
        raise ValueError(
            "Either epsilon or rho must be specified, and they are mutually exclusive."
        )  # pragma: no cover

    if epsilon is not None:
        _range_warning("epsilon", epsilon, 1, 5)
        measure, loss = max_divergence(), epsilon

    if rho is not None:
        _range_warning("rho", rho, 0.25, 0.5)
        measure, loss = zero_concentrated_divergence(), rho

    if delta is None:
        return measure, loss
    else:
        _range_warning("delta", delta, 1e-6, 1e-6)
        return approximate(measure), (loss, delta)


def unit_of(
    *,
    contributions: Optional[Union[int, Sequence[Bound]]] = None,
    changes: Optional[int] = None,
    absolute: Optional[float] = None,
    l1: Optional[float] = None,
    l2: Optional[float] = None,
    local: Optional[bool] = None,
    identifier: Optional[str] = None,
    ordered: bool = False,
    U=None,
) -> tuple[Metric, Union[float, Sequence[Bound]]]:
    """Constructs a unit of privacy, consisting of a metric and a dataset distance.
    The parameters are mutually exclusive.

    >>> import opendp.prelude as dp
    >>> dp.unit_of(contributions=3)
    (SymmetricDistance(), 3)
    >>> dp.unit_of(l1=2.0)
    (L1Distance(f64), 2.0)

    :param contributions: Greatest number of records or identifiers a privacy unit may contribute to microdata
    :param changes: Greatest number of records a privacy unit may change in microdata
    :param absolute: Greatest absolute distance a privacy unit can influence a scalar aggregate data set
    :param l1: Greatest l1 distance a privacy unit can influence a vector aggregate data set
    :param l2: Greatest l2 distance a privacy unit can influence a vector aggregate data set
    :param local: Set to true if the dataset consists of one individual (for local-DP)
    :param identifier: Can be a column name or Polars expression.
    :param ordered: Set to ``True`` to use ``InsertDeleteDistance`` instead of ``SymmetricDistance``, or ``HammingDistance`` instead of ``ChangeOneDistance``.
    :param U: The type of the dataset distance."""

    def _is_distance(p, v):
        return p not in ["ordered", "U", "_is_distance", "identifier"] and v is not None

    if sum(1 for p, v in locals().items() if _is_distance(p, v)) != 1:
        raise ValueError("Must specify exactly one distance.")

    if local:
        if identifier is not None or ordered or U is not None:
            raise ValueError('"local" must be the only parameter')
        return discrete_distance(), int(local)

    if ordered:
        if contributions is None and changes is None:
            raise ValueError(
                '"ordered" is only valid with "changes" or "contributions"'
            )
        if identifier is not None:
            raise ValueError('"ordered" must be False when "identifier" is set')

    if contributions is not None:
        if identifier is None:
            metric = insert_delete_distance() if ordered else symmetric_distance()
        else:
            metric = symmetric_id_distance(identifier)

        if isinstance(contributions, Sequence):
            metric = frame_distance(metric)

        return metric, contributions

    if changes is not None:
        if identifier is None:
            metric = hamming_distance() if ordered else change_one_distance()
        else:
            metric = change_one_id_distance(identifier)

        return metric, changes

    if absolute is not None:
        metric = absolute_distance(T=RuntimeType.parse_or_infer(U, absolute))
        return metric, absolute
    if l1 is not None:
        metric = l1_distance(T=RuntimeType.parse_or_infer(U, l1))
        return metric, l1
    if l2 is not None:
        metric = l2_distance(T=RuntimeType.parse_or_infer(U, l2))
        return metric, l2

    raise Exception("No matching metric found")  # pragma: no cover


class Context(object):
    """A Context coordinates queries to an instance of a privacy :py:attr:`accountant`.

    It is recommended to use :py:meth:`Context.compositor` constructor instead of this one.

    :param accountant: The measurement used to spawn the queryable.
    :param queryable: Executes the queries and tracks the privacy expenditure.
    :param d_in: An upper bound on the distance between adjacent datasets.
    :param d_mids: A sequence of privacy losses for each query to be sent to the queryable. Used for compositors.
    :param d_out: An upper bound on the overall privacy loss. Used for filters.
    :param query_space: The input space of queries to the context. If not specified, it defaults to the input space of the compositor.
    """

    accountant: Union[Measurement, Odometer]
    """The accountant is the measurement used to spawn the queryable.
    It contains information about the queryable, 
    such as the input domain, input metric, and output measure expected of measurement queries sent to the queryable."""
    queryable: Union[Queryable, OdometerQueryable]
    """The queryable executes the queries and tracks the privacy expenditure."""

    def __init__(
        self,
        accountant: Measurement,
        queryable: Queryable,
        d_in: Union[float, Sequence[Bound]],
        d_mids: Optional[Union[list[float], list[tuple[float, float]]]] = None,
        d_out: Optional[Union[float, tuple[float, float]]] = None,
        query_space: Optional[tuple[Domain, Metric]] = None,
    ):
        self.accountant = accountant
        self.queryable = queryable
        self.d_in = d_in
        self.d_mids = d_mids
        self.d_out = d_out

        self.d_mids_consumed: Union[list[float], list[tuple[float, float]]] = []
        self.query_space = query_space

    def __repr__(self) -> str:
        return f"""Context(
    accountant = {indent(repr(self.accountant))},
    d_in       = {self.d_in},
    d_mids     = {self.d_mids},
    d_out      = {self.d_out})"""

    def deserialize_polars_plan(self, serialized_plan: bytes) -> "LazyFrameQuery":
        """
        Given a serialized Polars plan, wraps it with a LazyFrameQuery.
        See the :ref:`serialization documentation <lazyframe-serialization>`
        for context and full example.

        :param serialized_plan: A plan like that returned by ``query.polars_plan.serialize()``
        """
        import io

        pl = import_optional_dependency("polars")

        new_plan = pl.LazyFrame.deserialize(io.BytesIO(serialized_plan))
        new_query = self.query()
        if not isinstance(new_query, LazyFrameQuery):
            raise ValueError("'data' of context must be a LazyFrame")

        new_query.polars_plan = new_plan
        return new_query

    @staticmethod
    def compositor(
        data: Any,
        privacy_unit: tuple[Metric, Union[float, Sequence[Bound]]],
        privacy_loss: tuple[Measure, Any],
        split_evenly_over: Optional[int] = None,
        split_by_weights: Optional[Sequence[float]] = None,
        domain: Optional[Domain] = None,
        margins: Optional[Sequence[Margin]] = None,
    ) -> "Context":
        """Constructs a new context containing a sequential compositor with the given weights.

        If the domain is not specified, it will be inferred from the data.
        This makes the assumption that the structure of the data is public information.

        ``split_evenly_over`` and ``split_by_weights`` are mutually exclusive.

        When ``data`` is a Polars LazyFrame, queries are specified as a Polars compute plan.
        In addition, ``margins`` may be specified, which contain descriptors for the data under grouping.

        :param data: The data to be analyzed.
        :param privacy_unit: The privacy unit of the compositor.
        :param privacy_loss: The privacy loss of the compositor.
        :param split_evenly_over: The number of parts to evenly distribute the privacy loss.
        :param split_by_weights: A list of weights for each intermediate privacy loss.
        :param domain: The domain of the data.
        :param margins: Descriptors for grouped data."""
        if domain is None:
            domain = domain_of(data, infer=True)

        if margins:
            # allows dictionaries of {[by]: [margin]}
            if isinstance(margins, MutableMapping):
                from warnings import warn

                warn(
                    'Margin dicts should be replaced with lists, with the key supplied as the "by" kwarg',
                    DeprecationWarning,
                )
                margins = [replace(margin, by=by) for by, margin in margins.items()]

            for margin in margins:
                domain = with_margin(domain, margin)

        accountant, d_mids, d_out = _normalize_compositor(
            domain, privacy_unit, privacy_loss, split_evenly_over, split_by_weights
        )

        try:
            queryable = accountant(data)
        except TypeError as e:
            inferred_domain = domain_of(data, infer=True)
            if vector_domain(domain) == inferred_domain:
                # With Python 3.11, add_note is available, but pytest.raises doesn't see notes.
                e.args = (
                    e.args[0] + "; To fix, wrap domain kwarg with dp.vector_domain()",
                )
            raise e

        return Context(
            accountant=accountant,
            queryable=queryable,
            d_in=privacy_unit[1],
            d_mids=d_mids,
            d_out=d_out,
        )

    def __call__(self, query: Measurement):
        """Executes the given query on the context."""
        answer = self.queryable(query)

        # pop won't fail if queryable returned an answer
        self.d_mids_consumed.append(
            query.map(self.d_in) if self.d_mids is None else self.d_mids.pop(0)  # type: ignore[arg-type]
        )
        return answer

    def query(self, **kwargs) -> Union["Query", LazyFrameQuery]:
        """Starts a new Query to be executed in this context.

        If the context has been constructed with a sequence of privacy losses,
        the next loss will be used. Otherwise, the loss will be computed from the kwargs.

        :param kwargs: The privacy loss to use for the query. Passed directly into :py:func:`loss_of`.
        """
        d_query = None
        if self.d_mids is not None:
            if kwargs:
                raise ValueError(
                    f"Expected no privacy arguments but got {kwargs}"
                )  # pragma: no cover
            if not self.d_mids:
                raise ValueError(
                    "Privacy allowance has been exhausted"
                )  # pragma: no cover
            d_query = self.d_mids[0]
        elif kwargs:
            observed_measure, d_query = loss_of(**kwargs)

            expected_measure = self.accountant.output_measure
            if observed_measure != expected_measure:
                msg = f"Expected output measure {expected_measure} but got {observed_measure}."
                if (
                    isinstance(expected_measure, ApproximateDivergence)
                    and "delta" not in kwargs
                ):
                    msg += " Consider setting `delta=0.0` in your query."
                raise ValueError(msg)

        chain = self.query_space or self.accountant.input_space
        query = Query(
            chain=chain,
            output_measure=self.accountant.output_measure,
            d_in=self.d_in,
            d_out=d_query,
            context=self,
        )

        # return a LazyFrameQuery when dealing with Polars data, to better mimic the Polars API
        if chain[0].type == "LazyFrameDomain":
            from opendp.domains import _lazyframe_from_domain

            # creates an empty lazyframe to hold the query plan
            polars_plan = _lazyframe_from_domain(self.accountant.input_domain)
            return LazyFrameQuery(polars_plan, query)

        return query

    def current_privacy_loss(self):
        """When the query losses are static, returns the list of consumed privacy losses,
        otherwise returns the current privacy loss of the queryable."""
        if isinstance(self.queryable, OdometerQueryable):
            return self.queryable.privacy_loss(self.d_in)
        return self.d_mids_consumed.copy()

    def remaining_privacy_loss(self):
        """Returns the remaining privacy loss."""
        if isinstance(self.queryable, OdometerQueryable):
            if self.d_out is None:
                raise ValueError("The privacy loss is unbounded.")

            def _sub(a, b):
                if (
                    isinstance(a, tuple)
                    and isinstance(b, tuple)
                    and len(a) == len(b) == 2
                ):
                    return a[0] - b[0], a[1] - b[1]
                else:
                    return a - b

            return _sub(self.d_out, self.queryable.privacy_loss(self.d_in))

        return None if self.d_mids is None else self.d_mids.copy()


Chain = Union[
    tuple[Domain, Metric], Transformation, Measurement, Odometer, "PartialChain"
]


class Query(object):
    """Initializes the query with the given chain and output measure.

    It is more convenient to use the ``context.query()`` constructor than this one.
    However, this can be used stand-alone to help build a transformation/measurement that is not part of a context.

    :param chain: an initial metric space (tuple of domain and metric) or transformation
    :param output_measure: how privacy will be measured on the output of the query
    :param d_in: an upper bound on the distance between adjacent datasets
    :param d_out: an upper bound on the overall privacy loss
    :param context: if specified, then when the query is released, the chain will be submitted to this context
    :param _wrap_release: for internal use only
    """

    _chain: Chain
    """The current chain of transformations and measurements."""
    _output_measure: Measure
    """The output measure of the query."""
    _context: Optional["Context"]
    """The context that the query is part of. ``query.release()`` submits ``_chain`` to ``_context``."""
    _wrap_release: Optional[Callable[[Any], Any]]
    """For internal use. A function that wraps the release of the query. 
    Used to wrap the response of compositor/odometer queries in another ``Context``."""

    def __init__(
        self,
        chain: Chain,
        output_measure: Measure,
        d_in: Optional[Union[float, Sequence[Bound]]] = None,
        d_out: Optional[Union[float, tuple[float, float]]] = None,
        context: Optional["Context"] = None,
        _wrap_release=None,
    ) -> None:
        self._chain = chain
        self._output_measure = output_measure
        self._d_in = d_in
        self._d_out = d_out
        self._context = context
        self._wrap_release = _wrap_release

    def __repr__(self) -> str:
        context = ""
        if self._context is not None:
            context = f""",\n    context        = {indent(repr(self._context))}"""

        return f"""Query(
    chain          = {indent(repr(self._chain))},
    output_measure = {self._output_measure},
    d_in           = {self._d_in},
    d_out          = {self._d_out}{context})"""

    def __getattr__(self, name: str) -> Callable[..., "Query"]:
        """Creates a new query by applying a transformation or measurement to the current chain."""
        if name == "canonical_noise":
            return self._canonical_noise

        if name not in constructors:
            raise AttributeError(
                f"Unrecognized constructor: '{name}'. Did you mean to '.release()' first?"
            )  # pragma: no cover

        def make(*args, **kwargs) -> "Query":
            """Wraps the ``make_{name}`` constructor to allow one optional parameter and chains it to the current query.

            This function will be called when the user calls ``query.{name}(...)``.

            :param args: arguments for the constructor function
            :param kwargs: keyword arguments for the constructor function
            """
            constructor, is_partial = constructors[name]

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
                raise ValueError(
                    f"{name} has {param_diff} parameter(s) too many."
                )  # pragma: no cover

            new_chain = constructor(*args, **kwargs)
            if is_partial or not isinstance(self._chain, tuple):
                new_chain = self._chain >> new_chain

            return self.new_with(chain=new_chain)

        return make

    def _canonical_noise(self, binomial_size: Optional[int] = None):
        """Make a measurement that adds noise from the canonical noise distribution.

        :param binomial_size: (approximate) total number of records if dataset consists of bernoulli draws
        """
        # left as private method so that it behaves like other constructors in IDEs
        # this is getting a special wrapper because
        # - its d_in argument is the stability of the prior transformation
        # - its d_out argument is already stored in the chain
        # - it has a special postprocessor
        from opendp.measurements import then_canonical_noise
        from opendp._internal import _new_pure_function
        from opendp.extras.numpy.canonical import BinomialCND

        def then(d_in, d_out):
            m_noise = then_canonical_noise(d_in, d_out)
            if binomial_size is not None:
                m_noise = m_noise >> _new_pure_function(
                    lambda x: BinomialCND(x, d_in, d_out, binomial_size),
                    TO="ExtrinsicObject",
                )
            return m_noise

        if isinstance(self._chain, tuple):
            d_mid = self._d_in
        elif isinstance(self._chain, Transformation):
            d_mid = self._chain.map(self._d_in)
        elif isinstance(self._chain, PartialChain):
            raise ValueError(
                "Canonical noise requires all arguments in the input query to be specified."
            )
        else:
            raise ValueError(  # pragma: no cover
                f"Canonical noise expects a metric space or transformation as the prior query, found {self._chain}"
            )

        return self.new_with(chain=self._chain >> then(d_mid, self._d_out))

    def new_with(self, *, chain: Chain, wrap_release=None) -> "Query":
        """Convenience constructor that creates a new query with a different chain.

        :param chain: the prior query. Either a metric space or transformation
        :param wrap_release: a function to apply to apply to releases
        """
        return Query(
            chain=chain,
            output_measure=self._output_measure,
            d_in=self._d_in,
            d_out=self._d_out,
            context=self._context,  # type: ignore[arg-type]
            _wrap_release=wrap_release or self._wrap_release,
        )

    def __dir__(self):
        """Returns the list of available constructors. Used by Python's error suggestion mechanism.
        Without this, none of the transformations or measument methods are listed.
        """
        return super().__dir__() + list(constructors.keys())  # type: ignore[operator]

    def resolve(
        self,
        allow_transformations: bool = False,
        bounds: Optional[tuple[float, float]] = None,
        T: Optional[Union[Type[float], Type[int]]] = None,
    ) -> Union[Transformation, Measurement, Odometer]:
        """Resolve the query into a transformation, measurement or odometer.

        :param allow_transformations: If true, allow the response to be a transformation.
        :param bounds: Search for a potentially missing parameter of interest within these bounds.
        :param T: The type of the parameter to search for. Either ``float`` or ``int``.
        """
        # resolve a partial chain into a measurement, by fixing the input and output distances
        if isinstance(self._chain, PartialChain):
            assert self._d_in is not None
            assert self._d_out is not None
            chain = self._chain.fix(
                self._d_in, self._d_out, self._output_measure, bounds=bounds, T=T
            )
        else:
            chain = self._chain
        if not allow_transformations and isinstance(chain, Transformation):
            raise ValueError("Query is not yet a measurement or odometer.")
        return _cast_measure(chain, self._output_measure, self._d_out)

    def release(
        self,
        data=None,
        bounds: Optional[tuple[float, float]] = None,
        T: Optional[Union[Type[float], Type[int]]] = None,
    ):
        """Release the query. The query must be part of a context.

        :param data: The data to be analyzed. If not specified, the query must be part of a context.
        :param bounds: Search for a potentially missing parameter of interest within these bounds.
        :param T: The type of the parameter to search for. Either ``float`` or ``int``.
        """
        if self._context is not None and data is not None:
            raise ValueError("Cannot specify data when the query is part of a context.")

        measurement = self.resolve(bounds=bounds, T=T)

        if self._context is not None:
            answer = self._context(measurement)
        elif data is not None:
            answer = measurement(data)
        else:
            raise ValueError("Cannot release query without data or context.")

        if self._wrap_release:
            answer = self._wrap_release(answer)

        return answer

    def param(
        self,
        allow_transformations=False,
        bounds: Optional[tuple[float, float]] = None,
        T: Optional[Union[Type[float], Type[int]]] = None,
    ):
        """Returns the discovered parameter, if there is one.

        :param allow_transformations: If true, allow the response to be a transformation.
        :param bounds: Search for a potentially missing parameter of interest within these bounds.
        :param T: The type of the parameter to search for. Either ``float`` or ``int``.
        """
        resolved = self.resolve(
            allow_transformations=allow_transformations, bounds=bounds, T=T
        )
        return getattr(resolved, "param", None)

    def compositor(
        self,
        split_evenly_over: Optional[int] = None,
        split_by_weights: Optional[Sequence[float]] = None,
        d_out: Optional[Union[float, tuple[float, float]]] = None,
        output_measure: Optional[Measure] = None,
        alpha: Optional[float] = None,
    ) -> "Query":
        """Constructs a new context containing a sequential compositor with the given weights.

        ``split_evenly_over`` and ``split_by_weights`` are mutually exclusive.

        :param split_evenly_over: The number of parts to evenly distribute the privacy loss
        :param split_by_weights: A list of weights for each intermediate privacy loss
        :param d_out: Optional upper bound on privacy loss.
        :param output_measure: Optional method of accounting to be used by this compositor. Defaults to same.
        :param alpha: Optional parameter to split delta between zCDP conversion and δ-approximate in approx-ZCDP
        """

        d_out = d_out or self._d_out

        if d_out is None:
            raise ValueError(
                "`d_out` is unknown. Please specify it in the query."
            )  # pragma: no cover

        if output_measure is not None:
            d_out = _translate_measure_distance(
                d_out, self._output_measure, output_measure, alpha
            )

        def _compositor(chain: Union[tuple[Domain, Metric], Transformation], d_in):
            if isinstance(chain, tuple):
                input_domain, input_metric = chain
            elif isinstance(chain, Transformation):
                input_domain, input_metric = chain.output_domain, chain.output_metric
                d_in = chain.map(d_in)

            privacy_unit = input_metric, d_in
            privacy_loss = output_measure or self._output_measure, d_out

            accountant, d_mids, _ = _normalize_compositor(
                input_domain,
                privacy_unit,
                privacy_loss,
                split_evenly_over,
                split_by_weights,
            )
            if isinstance(chain, Transformation):
                accountant = chain >> accountant

            def _wrap_release(queryable):
                return Context(
                    accountant=accountant,
                    queryable=queryable,
                    d_in=d_in,
                    d_mids=d_mids,
                    d_out=d_out,
                    query_space=(input_domain, input_metric),
                )

            return self.new_with(chain=accountant, wrap_release=_wrap_release)

        return self._compose_context(_compositor)

    def _compose_context(self, compositor):
        """Helper function for composition in a context."""
        if isinstance(self._chain, PartialChain):
            # TODO: Can we exercise this?
            return PartialChain(
                lambda x: compositor(self._chain(x), self._d_in)
            )  # pragma: no cover
        else:
            return compositor(self._chain, self._d_in)


class PartialChain(object):
    """A partial chain is a transformation or measurement that is missing one numeric parameter.

    The parameter can be solved for by calling the fix method,
    which returns the closest transformation or measurement that satisfies the given stability or privacy constraint.
    """

    partial: Callable[[float], Union[Transformation, Measurement]]
    """The partial transformation or measurement."""

    def __init__(self, f, *args, **kwargs):
        self.partial = partial(f, *args, **kwargs)

    def __call__(self, v):
        """Returns the transformation or measurement with the given parameter."""
        return self.partial(v)

    def fix(
        self,
        d_in: Union[float, Sequence[Bound]],
        d_out: Union[float, tuple[float, float]],
        output_measure: Optional[Measure] = None,
        bounds: Optional[tuple[float, float]] = None,
        T=None,
    ):
        """Returns the closest transformation or measurement that satisfies the given stability or privacy constraint.

        The discovered parameter is assigned to the param attribute of the returned transformation or measurement.

        :param d_in: Upper bound on the distance between adjacent input datasets.
        :param d_out: Upper bound on the distance between adjacent output distributions.
        :param output_measure: How to measure distances between output distributions.
        :param bounds: The bounds for the parameter search.
        :param T: The type of the parameter to search for.
        """
        # When the output measure corresponds to approx-DP, only optimize the epsilon parameter.
        # The delta parameter should be fixed in _cast_measure, and if not, then the search will be impossible here anyways.
        if (
            output_measure is not None
            and isinstance(output_measure.type, RuntimeType)
            and output_measure.type.origin == "Approximate"
        ):

            def _predicate(param):
                meas = _cast_measure(self(param), output_measure, d_out)
                return meas.map(d_in)[0] <= d_out[0]  # type: ignore[index]

        else:

            def _predicate(param):
                meas = _cast_measure(self(param), output_measure, d_out)
                return meas.check(d_in, d_out)

        param = binary_search(_predicate, bounds=bounds, T=T)
        chain = self.partial(param)
        chain.param = param
        return chain

    def __rshift__(
        self, other: Union[Transformation, Measurement, _PartialConstructor]
    ):
        # partials may be chained with other transformations or measurements to form a new partial
        if isinstance(other, (Transformation, Measurement, _PartialConstructor)):
            return PartialChain(lambda x: self(x) >> other)

        raise ValueError(
            "At most one parameter may be missing at a time"
        )  # pragma: no cover

    def __rrshift__(
        self, other: Union[tuple[Domain, Metric], Transformation, Measurement]
    ):
        if isinstance(other, (tuple, Transformation, Measurement)):
            return PartialChain(lambda x: other >> self(x))

        raise ValueError(
            "At most one parameter may be missing at a time"
        )  # pragma: no cover

    @classmethod
    def wrap(cls, f):
        """Wraps a constructor for a transformation or measurement to return a partial chain instead.

        :param f: function to wrap
        """

        def _inner(*args, **kwargs):
            return cls(f, *args, **kwargs)

        return _inner


def _normalize_compositor(
    domain: Domain,
    privacy_unit: tuple[Metric, Union[float, Sequence[Bound]]],
    privacy_loss: tuple[Measure, Union[float, tuple[float, float]]],
    split_evenly_over: Optional[int] = None,
    split_by_weights: Optional[Sequence[float]] = None,
) -> tuple[
    Union[Measurement, Odometer],
    Optional[Union[list[float], list[tuple[float, float]]]],
    Optional[Any],
]:
    """Constructs a sequential composition measurement
    where the ``d_mids`` are proportional to the weights.

    ``split_evenly_over`` and ``split_by_weights`` are mutually exclusive.

    * composition: returns a measurement, d_mids, and None
    * filter: returns a measurement, None, and d_out
    * adaptive: returns a measurement, None, and None

    :param domain: the domain of the data
    :param privacy_unit: a tuple of the input metric and the data distance (``d_in``)
    :param privacy_loss: a tuple of the output measure and the privacy parameter (``d_out``)
    :param split_evenly_over: The number of parts to evenly distribute the privacy loss
    :param split_by_weights: A list of weights for each intermediate privacy loss
    :return: a tuple of the measurement, d_mids, and d_out
    """
    input_metric, d_in = privacy_unit
    output_measure, d_out = privacy_loss

    if split_evenly_over is not None and split_by_weights is not None:
        raise ValueError(
            "Cannot specify both `split_evenly_over` and `split_by_weights`"
        )  # pragma: no cover

    if split_evenly_over is not None:
        weights = [d_out] * split_evenly_over
    elif split_by_weights is not None:
        # TODO: Wrap d_out in an object that defines __mul__,
        # and fix the typing on the signature.
        if isinstance(d_out, float):
            weights = [d_out * w for w in split_by_weights]
        else:
            weights = [(d_out[0] * w, d_out[1] * w) for w in split_by_weights]
    else:
        odometer = make_fully_adaptive_composition(
            input_domain=domain,
            input_metric=input_metric,
            output_measure=output_measure,
        )

        inf = float("inf")
        if d_out in {inf, (inf, inf), (inf, 1.0)}:
            return odometer, None, None

        return make_privacy_filter(odometer, d_in, d_out), None, d_out

    def _mul(dist, scale: float):
        if isinstance(dist, tuple):
            return dist[0] * scale, dist[1] * scale
        else:
            return dist * scale

    def _scale_weights(scale: float, weights):
        return [_mul(w, scale) for w in weights]

    def _scale_ac(scale: float):
        return make_adaptive_composition(
            input_domain=domain,
            input_metric=input_metric,
            output_measure=output_measure,
            d_in=d_in,
            d_mids=_scale_weights(scale, weights),
        )

    scale = binary_search_param(_scale_ac, d_in=d_in, d_out=d_out, T=float)

    # return the accountant and d_mids
    return _scale_ac(scale), _scale_weights(scale, weights), None


def _cast_measure(chain, to_measure: Optional[Measure] = None, d_to=None):
    """Casts the output measure of a given ``chain`` to ``to_measure``.

    If provided, ``d_to`` is the privacy loss wrt the new measure.
    """
    if to_measure is None or chain.output_measure == to_measure:
        return chain

    from_to = str(chain.output_measure.type), str(to_measure.type)

    if from_to == ("MaxDivergence", "Approximate<MaxDivergence>"):
        return make_approximate(chain)

    if from_to == (
        "ZeroConcentratedDivergence",
        "Approximate<ZeroConcentratedDivergence>",
    ):
        return make_approximate(chain)

    if from_to == ("MaxDivergence", "ZeroConcentratedDivergence"):
        return make_pureDP_to_zCDP(chain)

    if from_to == (
        "ZeroConcentratedDivergence",
        "Approximate<MaxDivergence>",
    ) or from_to == (
        "Approximate<ZeroConcentratedDivergence>",
        "Approximate<MaxDivergence>",
    ):
        return make_fix_delta(make_zCDP_to_approxDP(chain), d_to[1])

    raise ValueError(
        f"Unable to cast measure from {from_to[0]} to {from_to[1]}"
    )  # pragma: no cover


def _translate_measure_distance(
    d_from, from_measure: Measure, to_measure: Measure, alpha: Optional[float] = None
) -> Union[float, tuple[float, float]]:
    """Translate a privacy loss ``d_from`` from ``from_measure`` to ``to_measure``.

    >>> _translate_measure_distance(1, dp.max_divergence(), dp.max_divergence())
    1
    >>> _translate_measure_distance(1, dp.max_divergence(), dp.approximate(dp.max_divergence()))
    (1, 0.0)
    >>> _translate_measure_distance((1.5, 5e-07), dp.approximate(dp.max_divergence()), dp.zero_concentrated_divergence())
    0.0489...
    >>> _translate_measure_distance(0.05, dp.zero_concentrated_divergence(), dp.max_divergence())
    0.316...
    """
    if from_measure == to_measure:
        return d_from

    from_to = str(from_measure.type), str(to_measure.type)

    constant = 1.0  # the choice of constant doesn't matter

    if from_to == ("MaxDivergence", "Approximate<MaxDivergence>"):
        return (d_from, 0.0)

    if from_to == ("ZeroConcentratedDivergence", "MaxDivergence"):
        space = atom_domain(T=float, nan=False), absolute_distance(T=float)
        scale = binary_search_param(
            lambda scale: make_pureDP_to_zCDP(make_laplace(*space, scale)),
            d_in=constant,
            d_out=d_from,
            T=float,
        )
        return make_laplace(*space, scale).map(constant)

    if from_to == (
        "Approximate<MaxDivergence>",
        "ZeroConcentratedDivergence",
    ):

        def _caster(measurement):
            return make_fix_delta(make_zCDP_to_approxDP(measurement), delta=d_from[1])

        space = atom_domain(T=int), absolute_distance(T=float)
        scale = binary_search_param(
            lambda scale: _caster(make_gaussian(*space, scale)),
            d_in=constant,
            d_out=d_from,
            T=float,
        )
        return make_gaussian(*space, scale).map(constant)

    if from_to == (
        "Approximate<MaxDivergence>",
        "Approximate<ZeroConcentratedDivergence>",
    ):
        epsilon, delta = d_from
        if alpha is None or not (0 <= alpha < 1):
            raise ValueError(f"alpha ({alpha}) must be in [0, 1)")  # pragma: no cover
        delta_zCDP, delta_inf = delta * (1 - alpha), delta * alpha

        rho = _translate_measure_distance(
            (epsilon, delta_zCDP), from_measure, zero_concentrated_divergence()
        )
        return rho, delta_inf  # type: ignore[return-value]

    raise ValueError(
        f"Unable to translate distance from {from_to[0]} to {from_to[1]}"
    )  # pragma: no cover
