from opendp.extrinsics._utilities import register_transformation


def make_np_clamp(input_domain, input_metric, norm, p=2, origin=None):
    """Construct a new Transformation that clamps the norm of input data."""
    import opendp.prelude as dp
    import numpy as np # type: ignore[import]
    dp.assert_features("contrib")
    
    norm = float(norm)
    if norm < 0.: 
        raise ValueError("norm must not be negative")
    if p not in {1, 2}:
        raise ValueError("order p must be 1 or 2")

    if origin is None:
        origin = 0.0

    def function(arg):
        arg = arg.copy()
        arg -= origin

        # may have to run multiple times due to FP rounding
        current_norm = np.linalg.norm(arg, ord=p, axis=1, keepdims=True)
        while current_norm.max() > norm:
            arg /= np.maximum(current_norm / norm, 1)
            current_norm = np.linalg.norm(arg, ord=p, axis=1, keepdims=True)

        arg += origin
        return arg

    return dp.t.make_user_transformation(
        input_domain,
        input_metric,
        dp.np_array2_domain(**{**input_domain.descriptor, "norm": norm, "p": p, "origin": origin}),
        input_metric,
        function,
        lambda d_in: d_in,
    )


then_np_clamp = register_transformation(make_np_clamp)
