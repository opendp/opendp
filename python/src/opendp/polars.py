from opendp._lib import import_optional_dependency, lib_path

pl = import_optional_dependency("polars", raise_error=False)

if pl is not None:

    @pl.api.register_expr_namespace("dp")
    class DPExpr(object):
        def __init__(self, expr):
            self.expr = expr

        def noise(self, scale=None, distribution=None):
            """Add noise to the expression.

            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param scale: Noise scale parameter for the distribution.
            """
            return pl.plugins.register_plugin_function(
                plugin_path=lib_path,
                function_name="noise",
                kwargs={"scale": scale, "distribution": distribution},
                args=self.expr,
                is_elementwise=True,
            )

        def laplace(self, scale=None):
            """Add Laplace noise to the expression.

            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param scale: Noise scale parameter for the distribution. scale == standard_deviation / sqrt(2).
            """
            return self.noise(scale=scale, distribution="Laplace") # pragma: no cover

        def gaussian(self, scale=None):
            """Add Gaussian noise to the expression.

            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param scale: Noise scale parameter for the distribution. scale == standard_deviation.
            """
            return self.noise(scale=scale, distribution="Gaussian") # pragma: no cover

        def sum(self, bounds, scale=None):
            """Compute the differentially private sum.

            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param bounds: The bounds of the input data.
            :param scale: Noise scale parameter for the Laplace distribution. scale == standard_deviation / sqrt(2).
            """
            return self.expr.clip(*bounds).sum().dp.noise(scale) # pragma: no cover

        def mean(self, bounds, scale=None):
            """Compute the differentially private mean.

            The amount of noise to be added to the sum is determined by the scale.
            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param bounds: The bounds of the input data.
            :param scale: Noise scale parameter for the Laplace distribution. scale == standard_deviation / sqrt(2).
            """
            return self.expr.dp.sum(bounds, scale) / pl.len() # pragma: no cover

        def _discrete_quantile_score(self, alpha, candidates):
            """Score the utility of each candidate for representing the true quantile.

            Candidates closer to the true quantile are assigned scores closer to zero.
            Lower scores are better.

            :param alpha: a value in [0, 1]. Choose 0.5 for median
            :param candidates: Set of possible quantiles to evaluate the utility of.
            """
            return pl.plugins.register_plugin_function( # pragma: no cover
                plugin_path=lib_path,
                function_name="discrete_quantile_score",
                kwargs={"alpha": alpha, "candidates": candidates},
                args=self.expr,
                returns_scalar=True,
            )

        def _report_noisy_max_gumbel(self, optimize, scale=None):
            """Report the argmax or argmin after adding Gumbel noise.

            The scale calibrates the level of entropy when selecting an index.
            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param optimize: Distinguish between argmax and argmin.
            :param scale: Noise scale parameter for the Gumbel distribution.
            """
            return pl.plugins.register_plugin_function( # pragma: no cover
                plugin_path=lib_path,
                function_name="report_noisy_max_gumbel",
                kwargs={"optimize": optimize, "scale": scale},
                args=self.expr,
                is_elementwise=True,
            )

        def _index_candidates(self, candidates):
            """Index into a candidate set.

            Typically used after ``_report_noisy_max_gumbel`` to map selected indices to candidates.

            :param candidates: The values that each selected index corresponds to.
            """
            return pl.plugins.register_plugin_function( # pragma: no cover
                plugin_path=lib_path,
                function_name="index_candidates",
                kwargs={"candidates": candidates},
                args=self.expr,
                is_elementwise=True,
            )

        def quantile(self, alpha, candidates, scale=None):
            """Compute a differentially private quantile.

            The scale calibrates the level of entropy when selecting a candidate.
            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param alpha: a value in [0, 1]. Choose 0.5 for median
            :param candidates: Potential quantiles to select from.
            :param scale: How much noise to add to the scores of candidate.
            """
            dq_score = self.expr.dp._discrete_quantile_score(alpha, candidates) # pragma: no cover
            noisy_idx = dq_score.dp._report_noisy_max_gumbel("min", scale) # pragma: no cover
            return noisy_idx.dp._index_candidates(candidates) # pragma: no cover

        def median(self, candidates, scale=None):
            """Compute a differentially private median.

            The scale calibrates the level of entropy when selecting a candidate.
            If scale is None it is filled by ``global_scale`` in :py:func:`opendp.measurements.make_private_lazyframe`.

            :param candidates: Potential quantiles to select from.
            :param scale: How much noise to add to the scores of candidate.
            """
            return self.expr.dp.quantile(0.5, candidates, scale) # pragma: no cover
