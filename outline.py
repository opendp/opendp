
import polars as pl  # type: ignore[import-not-found]

columns = ["sports", "music", "travel"]
n_clusters = 8
model = KMeans(
    n_features=len(columns),
    n_clusters=n_clusters,
    scale=1.0,
    max_depth=6,
    lower=[0.0] * len(columns),
    upper=[1.0] * len(columns),
)

def make_to_array2d(columns):
    return dp.t.make_user_transformation(
        input_domain=IdGraphPersonObjectSetDomain(),
        input_metric=dp.symmetric_distance(),
        output_domain=dp.numpy.array2_domain(T=float, num_columns=len(columns)),
        output_metric=dp.symmetric_distance(),
        function=lambda people: extract_person_tag_matrix(people, columns),
        stability_map=lambda d_in: d_in,
    )

release = queryable(make_to_array2d(columns) >> model.measurement_)

def make_augmented_array2(release):
    t_predict = release.make_predict_transformation()
    t_dist = release.make_transform()

    return dp.t.make_user_transformation(
        input_domain=release.input_domain,
        input_metric=dp.symmetric_distance(),
        output_domain=dp.numpy.array2_domain(T=float, num_columns=release.n_features + 1),
        output_metric=dp.symmetric_distance(),
        # output columns are: original features..., cluster_id
        function=lambda x: _np().column_stack(
            [
                x,
                t_predict(x),
            ]
        ),
        stability_map=lambda d_in: d_in,
    )

# APPROACH 1: use polars
def make_clustered_lazyframe(release, columns):
    t_augmented = make_augmented_array2(release)

    def to_lazyframe(x):
        x = t_augmented(x)
        data = {
            **{name: x[:, j] for j, name in enumerate(columns)},
            "cluster": x[:, -1].astype(int),
        }
        return pl.LazyFrame(data)

    return dp.t.make_user_transformation(
        input_domain=release.input_domain,
        input_metric=dp.symmetric_distance(),
        output_domain=dp.lazyframe_domain(
            [
                *[
                    dp.series_domain(name, dp.atom_domain(T=float))
                    for name in columns
                ],
                dp.series_domain("cluster", dp.atom_domain(T=int)),
            ]
        ),
        output_metric=dp.symmetric_distance(),
        function=to_lazyframe,
        stability_map=lambda d_in: d_in,
    )

def make_polars_analysis(release, columns, exprs, scale):
    from opendp.domains import _lazyframe_from_domain

    t_clustered_lf = make_clustered_lazyframe(release, columns)

    plan = (
        _lazyframe_from_domain(t_clustered_lf.output_domain)
        .group_by("cluster")
        .agg(exprs)
        .join(pl.LazyFrame({"cluster": list(range(release.n_clusters))}))
    )

    return (
        make_to_array2d(columns)
        >> t_clustered_lf
        >> dp.m.make_private_lazyframe(
            input_domain=t_clustered_lf.output_domain,
            input_metric=dp.symmetric_distance(),
            output_measure=dp.max_divergence(),
            lazyframe=plan,
            global_scale=scale,
        )
    )

exprs = [
    dp.len().alias("members"),
    pl.col("sports").dp.mean((0, 1)),
]

# would need to tune scale
df = queryable(make_polars_analysis(release, columns, exprs, scale=1.0)).collect()

# APPROACH 2: use numpy
def make_partition_by_cluster(release):
    from opendp.extras.sklearn.cluster._tree import parallel_distance

    t_augmented = make_augmented_array2(release)
    feature_domain = dp.numpy.array2_domain(T=float, num_columns=release.n_features)

    return t_augmented >> dp.t.make_user_transformation(
        input_domain=t_augmented.output_domain,
        input_metric=dp.symmetric_distance(),
        output_domain=dp.vector_domain(feature_domain, size=release.n_clusters),
        # Conceptually this is an L0/Linf partition metric:
        # one changed row can affect at most one cluster partition (L0 = 1),
        # and within the affected partition the change is one row under the
        # child metric (Linf = d_in).
        output_metric=parallel_distance(dp.symmetric_distance()),
        function=lambda x: [x[x[:, -1] == cluster_id, :-1] for cluster_id in range(release.n_clusters)],
        stability_map=lambda d_in: (1, d_in),
    )

def make_numpy_group_sums(release, norm, p, scale):
    from opendp.extras.numpy._make_np_clamp import make_np_clamp
    from opendp.extras.numpy._make_np_sum import make_np_sum
    from opendp.extras.sklearn.cluster._tree import make_parallel_composition

    t_partitioned = make_partition_by_cluster(release)
    feature_domain = dp.numpy.array2_domain(T=float, num_columns=release.n_features)

    m_sums = make_parallel_composition(
        [
            (
                make_np_clamp(feature_domain, dp.symmetric_distance(), norm=norm, p=p)
                >> make_np_sum(feature_domain, dp.symmetric_distance())
                >> dp.m.then_noise(scale)
            )
            for _ in range(release.n_clusters)
        ]
    )

    return make_to_array2d(columns) >> t_partitioned >> m_sums

# Split into one block per cluster, then privately sum each block in parallel.
# May need to adjust norm and scale. p=2 means clamp to an l2 ball, which
# aligns with zCDP-style accounting.
sums = queryable(make_numpy_group_sums(release, norm=1.0, p=2, scale=1.0))
