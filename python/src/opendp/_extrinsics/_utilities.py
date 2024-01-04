from typing import Callable, Optional, Union
from opendp.mod import PartialConstructor, Measurement, Transformation


def to_then(func):
    """Convert any `make_` constructor to a `then_` constructor"""

    def then_func(*args, **kwargs):
        return PartialConstructor(
            lambda input_domain, input_metric: func(
                input_domain, input_metric, *args, **kwargs
            )
        )
    then_func.__name__ = func.__name__.replace("make_", "then_", 1)
    return then_func


def _register(module, constructor):
    import importlib, inspect

    module = importlib.import_module(f"opendp.{module}")
    module.__dict__[constructor.__name__] = constructor
    arg_names = inspect.getfullargspec(constructor)[0]

    if len(arg_names) >= 2 and arg_names[:2] == ["input_domain", "input_metric"]:
        then_const = to_then(constructor)
        module.__dict__[then_const.__name__] = then_const
        return then_const


def register_transformation(
    constructor: Callable[..., Transformation]
) -> Optional[PartialConstructor]:
    return _register("transformations", constructor)


def register_measurement(
    constructor: Callable[..., Measurement]
) -> Optional[PartialConstructor]:
    return _register("measurements", constructor)


def register_combinator(
    constructor: Callable[..., Union[Transformation, Measurement]],
) -> Optional[PartialConstructor]:
    return _register("combinators", constructor)


def with_privacy(t_constructor: Callable) -> Callable[..., Union[Transformation, Measurement]]:
    from opendp.mod import assert_features
    from opendp.measurements import then_gaussian, then_laplace
    def private_constructor(input_domain, input_metric, privacy_measure, scale):
        assert_features("contrib")
        m_constructor = {
            "ZeroConcentratedDivergence": then_gaussian,
            "MaxDivergence": then_laplace,
        }[privacy_measure.type.origin]

        return t_constructor(input_domain, input_metric) >> m_constructor(scale)

    private_constructor.__name__ = t_constructor.__name__.replace(
        "make_", "make_private_"
    )
    private_constructor.__doc__ = t_constructor.__doc__.replace(
        "Transformation", "Measurement"
    )
    return private_constructor

