:orphan:

.. code:: pycon

    # enable-features
    >>> import opendp.prelude as dp
    >>> import numpy as np
    >>> from pathlib import Path
    >>> dp.enable_features("contrib", "honest-but-curious")

    # /enable-features

    # pairwise-predict
    >>> def pairwise_predict(data, x_cuts):
    ...     data = np.array(data, copy=True)[: len(data) // 2 * 2]
    ...     np.random.shuffle(data)
    ...     p1, p2 = np.array_split(data, 2)
    ...     dx, dy = (p2 - p1).T
    ...     x_bar, y_bar = (p1 + p2).T / 2
    ...     points = dy / dx * (x_cuts[None].T - x_bar) + y_bar
    ...     return points.T[dx != 0]
    ...
    >>> def make_pairwise_predict(x_cuts, runs: int = 1):
    ...     return dp.t.make_user_transformation(
    ...         input_domain=dp.numpy.array2_domain(
    ...             num_columns=2, T=float
    ...         ),
    ...         input_metric=dp.symmetric_distance(),
    ...         output_domain=dp.numpy.array2_domain(
    ...             num_columns=2, T=float
    ...         ),
    ...         output_metric=dp.symmetric_distance(),
    ...         function=lambda x: np.vstack(
    ...             [
    ...                 pairwise_predict(x, x_cuts)
    ...                 for _ in range(runs)
    ...             ]
    ...         ),
    ...         stability_map=lambda d_in: d_in * runs,
    ...     )
    ...

    # /pairwise-predict

    # private-medians
    >>> def make_select_column(j):
    ...     return dp.t.make_user_transformation(
    ...         input_domain=dp.numpy.array2_domain(
    ...             num_columns=2, T=float
    ...         ),
    ...         input_metric=dp.symmetric_distance(),
    ...         output_domain=dp.vector_domain(
    ...             dp.atom_domain(T=float)
    ...         ),
    ...         output_metric=dp.symmetric_distance(),
    ...         function=lambda x: x[:, j],
    ...         stability_map=lambda d_in: d_in,
    ...     )
    ...
    >>> def make_private_percentile_medians(
    ...     output_measure, y_bounds, scale
    ... ):
    ...     m_median = dp.m.then_private_quantile(
    ...         output_measure,
    ...         candidates=np.linspace(*y_bounds, 100),
    ...         alpha=0.5,
    ...         scale=scale,
    ...     )
    ...     return dp.c.make_composition(
    ...         [
    ...             make_select_column(0)
    ...             >> dp.t.then_drop_null()
    ...             >> m_median,
    ...             make_select_column(1)
    ...             >> dp.t.then_drop_null()
    ...             >> m_median,
    ...         ]
    ...     )
    ...

    # /private-medians

    # mechanism
    >>> def make_private_theil_sen(
    ...     output_measure, x_bounds, y_bounds, scale, runs=1
    ... ):
    ...     x_cuts = x_bounds[0] + (
    ...         x_bounds[1] - x_bounds[0]
    ...     ) * np.array([0.25, 0.75])
    ...     p_inv = np.linalg.inv(
    ...         np.vstack([x_cuts, np.ones_like(x_cuts)]).T
    ...     )
    ...     return (
    ...         make_pairwise_predict(x_cuts, runs)
    ...         >> make_private_percentile_medians(
    ...             output_measure, y_bounds, scale
    ...         )
    ...         >> (lambda ys: p_inv @ ys)
    ...     )
    ...
    >>> x_bounds = (-3.0, 3.0)
    >>> y_bounds = (-10.0, 10.0)
    >>> meas = make_private_theil_sen(
    ...     dp.max_divergence(), x_bounds, y_bounds, scale=1.0
    ... )
    >>> meas.map(1)
    2.0

    # /mechanism

    # release
    >>> np.random.seed(1)
    >>> def f(x):
    ...     return x * 2 + 1
    ...
    >>> x = np.random.normal(size=100, loc=0, scale=1.0)
    >>> y = f(x) + np.random.normal(size=100, loc=0, scale=0.5)
    >>> slope, intercept = meas(np.stack([x, y], axis=1))
    >>> round(float(slope), 6), round(
    ...     float(intercept), 6
    ... )  # doctest: +ELLIPSIS
    (..., ...)

    # /release

.. code:: pycon

    # pairwise-visualization
    >>> def render_pairwise_visualization():
    ...     import matplotlib.pyplot as plt
    ...     pairwise_plot_path = Path(__file__).with_name(
    ...         "theil-sen-pairwise-fits.png"
    ...     )
    ...     pairwise_plot_x_cuts = np.array([-1.5, 1.5])
    ...     pairwise_plot_x = np.linspace(-2.5, 2.5, 12)
    ...     pairwise_plot_y = 2 * pairwise_plot_x + 1
    ...     pairwise_plot_data = np.column_stack(
    ...         [pairwise_plot_x, pairwise_plot_y]
    ...     )
    ...     p1, p2 = np.array_split(pairwise_plot_data, 2)
    ...     fig, ax = plt.subplots(figsize=(7, 4.5))
    ...     ax.scatter(
    ...         pairwise_plot_data[:, 0],
    ...         pairwise_plot_data[:, 1],
    ...         color="black",
    ...         s=20,
    ...     )
    ...     for point_1, point_2 in zip(p1, p2):
    ...         slope = (point_2[1] - point_1[1]) / (
    ...             point_2[0] - point_1[0]
    ...         )
    ...         intercept = point_1[1] - slope * point_1[0]
    ...         ax.plot(
    ...             pairwise_plot_x,
    ...             slope * pairwise_plot_x + intercept,
    ...             color="#7a9e9f",
    ...             alpha=0.45,
    ...         )
    ...         predicted = slope * pairwise_plot_x_cuts + intercept
    ...         ax.scatter(
    ...             pairwise_plot_x_cuts,
    ...             predicted,
    ...             color="#bc4b51",
    ...             s=28,
    ...             zorder=3,
    ...         )
    ...     for x_cut in pairwise_plot_x_cuts:
    ...         ax.axvline(
    ...             x_cut,
    ...             color="#2c4c7c",
    ...             linestyle="--",
    ...             linewidth=1,
    ...         )
    ...     ax.set_title("Pairwise Predictions at Fixed Cut Points")
    ...     ax.set_xlabel("x")
    ...     ax.set_ylabel("y")
    ...     fig.tight_layout()
    ...     fig.savefig(pairwise_plot_path, dpi=150)
    ...     plt.close(fig)
    ...

    # /pairwise-visualization

    # private-fit-visualization
    >>> def render_private_fit_visualization():
    ...     import matplotlib.pyplot as plt
    ...     private_fit_plot_path = Path(__file__).with_name(
    ...         "theil-sen-private-fit.png"
    ...     )
    ...     fig, ax = plt.subplots(figsize=(7, 4.5))
    ...     ax.scatter(
    ...         x,
    ...         y,
    ...         color="black",
    ...         s=14,
    ...         alpha=0.7,
    ...         label="synthetic data",
    ...     )
    ...     plot_x = np.linspace(x.min(), x.max(), 200)
    ...     ax.plot(
    ...         plot_x,
    ...         f(plot_x),
    ...         color="#2c4c7c",
    ...         linewidth=2,
    ...         label="true fit",
    ...     )
    ...     ax.plot(
    ...         plot_x,
    ...         slope * plot_x + intercept,
    ...         color="#bc4b51",
    ...         linewidth=2,
    ...         label="private fit",
    ...     )
    ...     ax.set_title("Private Theil-Sen Regression Fit")
    ...     ax.set_xlabel("x")
    ...     ax.set_ylabel("y")
    ...     ax.legend()
    ...     fig.tight_layout()
    ...     fig.savefig(private_fit_plot_path, dpi=150)
    ...     plt.close(fig)
    ...

    # /private-fit-visualization
