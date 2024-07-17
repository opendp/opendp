from __future__ import annotations
from typing import Optional, Union, Literal
from dataclasses import dataclass, field

import opendp.prelude as dp
from opendp.mod import Domain, Metric, Measurement, assert_features
from opendp._lib import import_optional_dependency, get_np_csprng
from opendp._extrinsics.synth.base import Synthesizer

import numpy as np
import polars as pl

np_csprng = get_np_csprng()


@dataclass
class Schema:
    bounds: dict[str, tuple[int, int]]
    size: int
    dict_schema: dict[str, type] = field(init=False)
    lf_schema: pl.LazyFrame = field(init=False)

    def __post_init__(self):
        self.dict_schema = {col: pl.Int32 for col in self.bounds.keys()}
        self.lf_schema = pl.LazyFrame(schema=self.dict_schema)


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

    def plan(self, data: Optional[Union[pl.DataFrame, pl.LazyFrame]] = None) -> Union[pl.DataFrame, pl.LazyFrame]:
        if data is None:
            return (self.schema.lf_schema
                    .filter(pl.col(self.column) == self.value)
                    .select(pl.len().dp.noise()))
        else:
            return (data.select(self.column)
                    .filter(pl.col(self.column) == self.value)
                    .count())

    def apply(self, data: Union[pl.DataFrame, pl.LazyFrame, np.ndarray]) -> Union[pl.DataFrame, pl.LazyFrame, np.ndarray]:
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
        return (self.column_index, self.value_index, self.column, self.value) == (other.column_index, other.value_index, other.column, other.value)

    def __repr__(self):
        return f"SimpleLinearQuery(column_index={self.column_index}, value_index={self.value_index}, column={self.column}, value={self.value})"


class MWEMSynthesizer(Synthesizer):

    def __init__(self,
                 input_domain: Domain,
                 input_metric: Metric,
                 schema: Schema,
                 epsilon: float,
                 epsilon_split: float,
                 num_queries: int,
                 num_iterations: int,
                 num_mult_weights_iterations: int,
                 verbose: bool = False):

        super().__init__()

        assert_features("contrib", "floating-point")

        if "LazyFrameDomain" not in str(input_domain.type):
            raise ValueError("input_domain must be a LazyFrame domain")

        if input_metric != dp.symmetric_distance():
            raise ValueError("input metric must be symmetric distance")

        self.input_domain = input_domain
        self.input_metric = input_metric

        self.schema = schema

        self.epsilon = epsilon
        self.epsilon_split = epsilon_split

        self.num_queries = num_queries
        self.num_iterations = num_iterations
        self.num_mult_weights_iterations = num_mult_weights_iterations

        self.distributions = None
        self.d_in = 1

        self.verbose = verbose
        if verbose:
            self.tqdm = import_optional_dependency("tqdm")

    def fit(self, data: pl.LazyFrame):
        super().fit(data)

        epsilon_per_iteration = self.epsilon / self.num_iterations
        epsilon_select = self.epsilon_split * epsilon_per_iteration
        epsilon_measure = epsilon_per_iteration - epsilon_select

        initial_distribution, dimensions, query_collection = self._setup()

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

        self.distributions = distributions
        self.dimensions = dimensions

    def _iteration(self,
                        comp,
                        epsilon_select: float,
                        epsilon_measure: float,
                        distribution: np.ndarray,
                        query_collection: list[SimpleLinearQuery]) -> np.ndarray:

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

    def _setup(self) -> tuple[np.ndarray, list[np.ndarray], list[SimpleLinearQuery]]:
        initial_distribution = np.ones([upper - lower + 1
                                        for lower, upper in self.schema.bounds.values()])
        initial_distribution /= initial_distribution.size

        dimensions = [np.arange(lower, upper + 2, dtype=int)
                      for lower, upper in self.schema.bounds.values()]

        query_collection = list({SimpleLinearQuery.random(self.schema)
                                 for _ in range(self.num_queries)})

        return initial_distribution, dimensions, query_collection

    def _select(self,
                epsilon: float,
                distribution: np.ndarray,
                query_collection: list[SimpleLinearQuery]) -> Measurement:

        def score_function(data):
            return [np.abs(query.apply(data).collect().item() - query.apply(distribution))
                    for query in query_collection]

        scores_trans = dp.t.make_user_transformation(
            input_domain=self.input_domain,
            input_metric=self.input_metric,
            output_domain=dp.vector_domain(dp.atom_domain(T=float)),
            output_metric=dp.linf_distance(T=float),
            function=score_function,
            stability_map=lambda d_in: float(d_in)
        )

        max_index_meas = dp.binary_search_chain(
            lambda s: scores_trans 
            >> dp.m.then_report_noisy_max_gumbel(
                scale=s,
                optimize="max"),
            self.d_in,
            epsilon,
        )

        return max_index_meas

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
                last_distribution: np.ndarray,
                query: SimpleLinearQuery,
                measurment: float) -> np.ndarray:
        new_distribution = last_distribution.copy()

        for _ in range(self.num_mult_weights_iterations):
            weights = ((measurment - query.apply(new_distribution))
                       / (2 * self.schema.size))
            new_distribution *= weights

            new_distribution /= new_distribution.sum()
        return new_distribution

    def sample(self,
               num_samples: int,
               agg_method: Union[Literal["last", "avg"]] = "last") -> pl.DataFrame:
        super().sample(num_samples)

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

    def releasable(self) -> list[np.ndarray]:
        super().releasable()
        return self.distributions
