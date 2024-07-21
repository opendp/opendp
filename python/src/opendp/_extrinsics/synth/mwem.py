from __future__ import annotations
from typing import Any, Union, Literal
from dataclasses import dataclass, field

import opendp.prelude as dp
from opendp.mod import Domain, Metric, Measurement
from opendp._lib import import_optional_dependency, get_np_csprng
from opendp._extrinsics.synth.base import SynthesizerTrainer, ReleasedSynthesizer

np_csprng = get_np_csprng()


@dataclass
class Schema:
    bounds: dict[str, tuple[int, int]]
    size: int
    dict_schema: dict[str, type] = field(init=False)
    lf_schema: Any = field(init=False)
    dims: Any = field(init=False)

    def __post_init__(self):
        pl = import_optional_dependency("polars")
        self.dict_schema = {col: pl.Int32 for col in self.bounds.keys()}
        self.lf_schema = pl.LazyFrame(schema=self.dict_schema)
        self.dims = [upper - lower + 1
                     for lower, upper in self.bounds.values()]


class SimpleLinearQuery:
    def __init__(self,
                 schema: Schema,
                 column_index: int,
                 value_index: int,
                 column: str,
                 value: int):
        self.schema = schema

        self.column_index = column_index
        self.value_index = value_index
        self.column = column
        self.value = value

    def stability_map(self, d_in):
        return float(d_in)

    def mask(self):
        np = import_optional_dependency("numpy")

        mask = np.zeros(self.schema.dims, dtype=int)
        mask.take(indices=self.value_index, axis=self.column_index)[:] = 1
        return mask

    def plan(self, data=None):
        pl = import_optional_dependency("polars")
        if data is None:
            return (self.schema.lf_schema
                    .filter(pl.col(self.column) == self.value)
                    .select(pl.len().dp.noise()))
        else:
            return (data.select(self.column)
                    .filter(pl.col(self.column) == self.value)
                    .count())

    def apply(self, data):
        np = import_optional_dependency("numpy")
        pl = import_optional_dependency("polars")

        if isinstance(data, (pl.DataFrame, pl.LazyFrame)):
            return self.plan(data)

        elif isinstance(data, np.ndarray):
            np.testing.assert_allclose(data.sum(), 1)
            marginal_axes = tuple(i for i in range(data.ndim)
                                  if i != self.column_index)
            marginal_distribution = data.sum(axis=marginal_axes)
            marginal_count = marginal_distribution[self.value_index] * self.schema.size
            return marginal_count

        else:
            raise ValueError(f"Unsupported data type: {type(data)}")

    @staticmethod
    def random(schema: Schema) -> SimpleLinearQuery:
        column_index = np_csprng.integers(0, len(schema.bounds))
        column = list(schema.bounds.keys())[column_index]
        lower, upper = schema.bounds[column]
        value_index = np_csprng.integers(0, upper - lower + 1)
        value = lower + value_index
        return SimpleLinearQuery(schema, column_index, value_index, column, value)

    def __hash__(self):
        return hash((self.column_index, self.value_index, self.column, self.value))

    def __eq__(self, other):
        return ((self.column_index, self.value_index, self.column, self.value)
                == (other.column_index, other.value_index, other.column, other.value))

    def __repr__(self):
        return f"SimpleLinearQuery(column_index={self.column_index}, value_index={self.value_index}, column={self.column}, value={self.value})"


class MWEMSynthesizerTrainer(SynthesizerTrainer):

    def __init__(self,
                 input_domain: Domain,
                 input_metric: Metric,
                 epsilon: float,
                 schema: Schema,
                 epsilon_split: float,
                 num_queries: int,
                 num_iterations: int,
                 num_mult_weights_iterations: int,
                 verbose: bool = False):

        super().__init__(input_domain, input_metric, epsilon)

        self.schema = schema

        self.epsilon_split = epsilon_split
        self.num_queries = num_queries
        self.num_iterations = num_iterations
        self.num_mult_weights_iterations = num_mult_weights_iterations

        self.d_in = 1

        self.verbose = verbose
        if verbose:
            self.tqdm = import_optional_dependency("tqdm")

    def fit(self, data):

        epsilon_per_iteration = self.epsilon / self.num_iterations
        epsilon_select = self.epsilon_split * epsilon_per_iteration
        epsilon_measure = epsilon_per_iteration - epsilon_select

        initial_distribution, query_collection = self._setup()

        distributions = [initial_distribution]

        mwem = dp.c.make_sequential_composition(
             self.input_domain,
             self.input_metric,
             dp.max_divergence(T=float),  # TODO??
             self.d_in,
             [epsilon_select, epsilon_measure] * self.num_iterations
        )

        comp = mwem(data)
        del data

        step_iter = self.tqdm.trange if self.verbose else range

        for _ in step_iter(self.num_iterations):

            last_distribution = distributions[-1]

            new_distribution = self._iteration(comp,
                                               epsilon_select,
                                               epsilon_measure,
                                               last_distribution,
                                               query_collection)

            distributions.append(new_distribution)

        configuration = {
            "epsilon_split": self.epsilon_split,
            "num_queries": self.num_queries,
            "num_iterations": self.num_iterations,
            "num_mult_weights_iterations": self.num_mult_weights_iterations,
        }
        return ReleasedMWEMSynthesizer(self.input_domain,
                                       self.input_metric,
                                       self.epsilon,
                                       configuration,
                                       self.schema,
                                       distributions)

    def _iteration(self,
                   comp,
                   epsilon_select: float,
                   epsilon_measure: float,
                   distribution,
                   query_collection: list[SimpleLinearQuery]):

        selected_query_index = comp(
            self._select(
                epsilon_select,
                distribution,
                query_collection
            )
        )

        selected_query = query_collection[selected_query_index]

        selected_query_measurement = comp(
            self._measure(
                epsilon_measure,
                selected_query
            )
        ).collect().item()

        new_distribution = self._update(distribution,
                                        selected_query,
                                        selected_query_measurement)

        return new_distribution

    def _setup(self):
        np = import_optional_dependency("numpy")

        initial_distribution = np.ones(self.schema.dims)
        initial_distribution /= initial_distribution.size

        query_collection = list({SimpleLinearQuery.random(self.schema)
                                 for _ in range(self.num_queries)})

        return initial_distribution, query_collection

    def _score(self,
               distribution,
               query_collection: list[SimpleLinearQuery]) -> dp.Transformation:

        np = import_optional_dependency("numpy")
        # TODO: consider using a more efficient implementation after profiling

        def function(data):
            return [np.abs(query.apply(data).collect().item()
                           - query.apply(distribution))
                    for query in query_collection]

        return dp.t.make_user_transformation(
            input_domain=self.input_domain,
            input_metric=self.input_metric,
            output_domain=dp.vector_domain(dp.atom_domain(T=float)),
            output_metric=dp.linf_distance(T=float),
            function=function,
            stability_map=lambda d_in: max(query.stability_map(d_in) for query in query_collection)
        )

    def _select(self,
                epsilon: float,
                distribution,
                query_collection: list[SimpleLinearQuery]) -> Measurement:

        return dp.binary_search_chain(
            lambda s: self._score(distribution, query_collection)
            >> dp.m.then_report_noisy_max_gumbel(
                scale=s,
                optimize="max"),
            self.d_in,
            epsilon,
        )

    def _measure(self,
                 epsilon: float,
                 query: SimpleLinearQuery) -> Measurement:

        meas = dp.binary_search_chain(
            lambda s: dp.make_private_lazyframe(
                self.input_domain,
                self.input_metric,
                dp.max_divergence(T=float),
                query.plan(),
                global_scale=s),
            self.d_in,
            epsilon)

        return meas

    def _update(self,
                last_distribution,
                query: SimpleLinearQuery,
                measurment: float):

        np = import_optional_dependency("numpy")

        distribution = last_distribution.copy()

        for _ in range(self.num_mult_weights_iterations):
            error = measurment - query.apply(distribution)

            multiplicative_weights = np.exp(
                query.mask() * error
                / (2 * self.schema.size)
            )
            multiplicative_weights[multiplicative_weights == 0.0] = 1.0

            distribution *= multiplicative_weights
            distribution /= distribution.sum()

        return distribution

    # OpenDP style make_private_... function
    @classmethod
    def make(cls,
             input_domain: Domain,
             input_metric: Metric,
             epsilon: float,
             *args, **kwargs):

        assert cls is MWEMSynthesizerTrainer

        synthesizer = cls(input_domain,
                          input_metric,
                          epsilon,
                          *args, **kwargs)

        return dp.m.make_user_measurement(
            input_domain,
            input_metric,
            dp.max_divergence(T=float),
            synthesizer.fit,
            lambda d_in: synthesizer.epsilon * d_in
        )


class ReleasedMWEMSynthesizer(ReleasedSynthesizer):
    def __init__(self,
                 input_domain: Domain,
                 input_metric: Metric,
                 epsilon: float,
                 configuation: dict,
                 schema: Schema,
                 distributions: list,
                 ):
        self.input_domain = input_domain
        self.input_metric = input_metric
        self.epsilon = epsilon
        self.configuation = configuation
        self.schema = schema
        self.distributions = distributions

    def sample(self,
               num_samples: int,
               agg_method: Union[Literal["last", "avg"]] = "last"):

        np = import_optional_dependency("numpy")
        pl = import_optional_dependency("polars")

        match agg_method:
            case "last":
                distribution = self.distributions[-1]
            case "avg":
                distribution = np.mean(self.distributions, axis=0)
                distribution /= distribution.sum()
            case _:
                raise ValueError(f"Unsupported aggregation method: {agg_method}")

        flat_distribution = distribution.flatten()

        sampled_flat_indices = np_csprng.choice(len(flat_distribution),
                                                p=flat_distribution,
                                                size=num_samples)

        sampled_indices = np.unravel_index(sampled_flat_indices, distribution.shape)

        synth_df = (pl.DataFrame(sampled_indices,
                                 schema=self.schema.dict_schema)
                    .with_columns(
                        [(pl.col(col) + lower).alias(col)
                         for col, (lower, _) in self.schema.bounds.items()])
                    )

        return synth_df
