from typing import Callable, Union

from opendp.mod import Domain, Metric, Transformation
import opendp.prelude as dp
from opendp._lib import import_optional_dependency
from opendp._extrinsics._utilities import to_then, with_privacy


__all__ = ["make_faithfulness",
           "then_faithfulness",
           "make_private_faithfulness",
           "then_private_faithfulness"]


def make_faithfulness(
        input_domain: Domain,
        input_metric: Metric,
        *,
        reference_dataset,
        similarity: Union[Callable, dict[str, float]]) -> Transformation:
    r"""Construct a Transformation that returns the optimal (maximal) faithfulness of two datasets.
    Faithfulness assesses whether the released dataset resemble the original dataset in a record-level granularity.
    Namely, there is a 1-to-1 matching between the records of released dataset and the records of the original dataset: each released record is matched with an original record.
    Two records are considered similar if their feature space distance, according to the similarity function, is less than or equal to 1.

    Presented in [Israel's Birth Dataset release](https://arxiv.org/pdf/2405.00267#page=13).

    :param input_domain: instance of `LazyFrameDomain`
    :param input_metric: instance of `symmetric_distance()`
    :param reference_dataset: public dataset to compare against, instance of `LazyFrame` or `DataFrame`
    :param similarity: function measuing the similarity between two records
            or a dictionary of weights for each column scaling the absolute difference

    :returns a Transformation that computes a tuple of (mean, S, Vt)
    """

    np = import_optional_dependency("numpy")
    pl = import_optional_dependency("polars")
    scipy = import_optional_dependency("scipy")
    sklearn_neighbors = import_optional_dependency("sklearn.neighbors")
    igraph = import_optional_dependency("igraph")

    dp.assert_features("contrib", "floating-point")

    if "LazyFrameDomain" not in str(input_domain.type):
        raise ValueError("input_domain must be a LazyFrame domain")

    # TODO: how to enfornce public knowlege of dataset size, if at all?
    # if (input_domain.descriptor.get("size")) is None:
    #     raise ValueError("input_domain's size must be known")

    # assert input_domain.member(reference_dataset.lazy())

    if input_metric != dp.symmetric_distance():
        raise ValueError("input metric must be symmetric distance")

    if callable(similarity):
        similarity_metric = similarity

    elif isinstance(similarity, dict):
        assert reference_dataset.columns == list(similarity.keys())
        metric_weights = np.array(list(similarity.values()))

        def similarity_metric(xi, yj):
            return np.sum(np.abs((xi - yj)) * metric_weights)

    else:
        raise ValueError("similarity must be a callable or a dictionary")

    def similarity_fn(X, Y):
        # TODO: consider using the underline scipy implementation directly
        neigh = sklearn_neighbors.NearestNeighbors(metric=similarity_metric, p=1, radius=1)
        neigh.fit(X)
        similar_record_indices = neigh.radius_neighbors(Y, return_distance=False)
        return similar_record_indices

    def _compute_optimal_faithfulness_cardinality(dataset):        
        num_records = len(reference_dataset)

        similar_record_indices = similarity_fn(dataset.collect(), reference_dataset)

        # build bipartite graph and find maximum matching

        gmat = scipy.sparse.lil_matrix((num_records, num_records))

        for row, cols in enumerate(similar_record_indices):
            if len(cols):
                gmat[row, sorted(cols)] = np.ones(len(cols), dtype=int)

        gmat = gmat.tocsr()

        nz = gmat.nonzero()

        edges = zip(nz[0], nz[1] + num_records)

        bipartite_mask = [False] * num_records + [True] * num_records
        g = igraph.Graph.Bipartite(bipartite_mask, edges, directed=False)

        matched_vertices = np.array(g.maximum_bipartite_matching().matching)[:num_records]

        second_matched_mask = matched_vertices[:num_records] != -1
        second_matched = np.nonzero(second_matched_mask)[0]
        first_matched = matched_vertices[second_matched] - num_records

        assert len(first_matched) == len(second_matched)
        matching_cardinality = len(first_matched)

        # matching = (first_matched, second_matched)
        # vertex_indices = set(range(num_records))
        # unmatched_indices = (vertex_indices - set(first_matched),
        #                       vertex_indices - set(second_matched)
        # )

        return matching_cardinality

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.atom_domain(T=int),
        dp.absolute_distance(T=int),
        _compute_optimal_faithfulness_cardinality,
        lambda d_in: d_in,
    )


then_faithfulness = to_then(make_faithfulness)
make_private_faithfulness = with_privacy(make_faithfulness)
then_private_faithfulness = to_then(make_private_faithfulness)
