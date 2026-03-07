from typing import Callable, Union
from opendp.mod import Domain, Metric, _PartialConstructor, Measurement, Transformation
import inspect


def supports_partial(constructor: Callable[..., Union[Transformation, Measurement]]) -> bool:
    """Check if the constructor can be written as a then_ (partial application) function.

    This is true if the first two arguments are input_domain and input_metric.
    """
    arg_names = inspect.getfullargspec(constructor)[0]
    return len(arg_names) > 1 and arg_names[:2] == ["input_domain", "input_metric"]


def to_then(constructor) -> Callable[..., _PartialConstructor]:
    """Convert any `make_` constructor to a `then_` constructor"""

    if not supports_partial(constructor):  # pragma: no cover
        raise ValueError("the first two arguments of the constructor must be input_domain and input_metric")
    
    def then_func(*args, **kwargs):
        return _PartialConstructor(
            lambda input_domain, input_metric: constructor(
                input_domain, input_metric, *args, **kwargs
            )
        )

    then_func.__name__ = constructor.__name__.replace("make_", "then_", 1)
    then_func.__doc__ = f"""{constructor.__doc__}

    Fixes the ``input_domain`` and ``input_metric`` of: ``{constructor.__name__}``"""

    # preserve the signature, except for the first two parameters
    original_sig = inspect.signature(constructor)
    new_params = list(original_sig.parameters.values())[2:]
    then_func.__signature__ = original_sig.replace(parameters=new_params) # type: ignore[attr-defined]
    return then_func


def with_privacy(
    t_constructor: Callable[[Domain, Metric], Transformation],
) -> Callable[..., Union[Transformation, Measurement]]:
    from opendp.mod import assert_features
    from opendp.measurements import then_gaussian, then_laplace

    def private_constructor(input_domain, input_metric, privacy_measure, scale,
                            *args, **kwargs):
        assert_features("contrib")
        m_constructor = {
            "ZeroConcentratedDivergence": then_gaussian,
            "MaxDivergence": then_laplace,
        }[str(privacy_measure.type)]

        return (t_constructor(input_domain, input_metric, *args, **kwargs)
                >> m_constructor(scale)) # type: ignore[operator]

    private_constructor.__name__ = t_constructor.__name__.replace(
        "make_", "make_private_"
    )
    if t_constructor.__doc__ is not None:
        private_constructor.__doc__ = t_constructor.__doc__.replace(
            "Transformation", "Measurement"
        )
    return private_constructor
