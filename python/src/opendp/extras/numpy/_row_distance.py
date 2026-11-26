"""Distance maps for conversions between Polars frames and NumPy row arrays."""


def _row_distance(metric):
    """Preserve SymmetricDistance for row-aligned conversions."""
    from opendp.metrics import symmetric_distance

    if metric != symmetric_distance():
        raise ValueError("Conversion requires SymmetricDistance")
    return metric, lambda d: d


def _frame_distance_to_symmetric_distance(metric):
    """Lower a global Polars frame contribution bound to a row distance."""
    from opendp.metrics import _get_bound, symmetric_distance
    from opendp.mod import FrameDistance

    if (
        not isinstance(metric, FrameDistance)
        or metric.inner_metric != symmetric_distance()
    ):
        raise ValueError(
            "Only FrameDistance<SymmetricDistance> can lower to row distance"
        )

    def stability(bounds):
        bound = _get_bound(bounds, []).per_group
        if bound is None:
            raise ValueError("A global row-contribution bound is required")
        return bound

    return symmetric_distance(), stability
