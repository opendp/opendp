from typing import Callable, Union
from opendp.mod import Domain, Metric, PartialConstructor, Measurement, Transformation


def to_then(constructor) -> Callable[..., PartialConstructor]:
    """Convert any `make_` constructor to a `then_` constructor"""

    def then_func(*args, **kwargs):
        return PartialConstructor(
            lambda input_domain, input_metric: constructor(
                input_domain, input_metric, *args, **kwargs
            )
        )

    then_func.__name__ = constructor.__name__.replace("make_", "then_", 1)
    then_func.__doc__ = f"partial constructor of {constructor.__name__}"
    return then_func


def _register(module_name, constructor) -> Callable[..., PartialConstructor]:  # type: ignore[return]
    import importlib
    import inspect

    module = importlib.import_module(f"opendp.{module_name}")
    setattr(module, constructor.__name__, constructor)
    arg_names = inspect.getfullargspec(constructor)[0]

    if len(arg_names) >= 2 and arg_names[:2] == ["input_domain", "input_metric"]:
        then_const = to_then(constructor)
        then_const.__doc__ = f"""{then_const.__doc__}

        Fixes the `input_domain` and `input_metric` of:
        :py:func:`opendp.{module_name}.{constructor.__name__}`"""
        setattr(module, then_const.__name__, then_const)
        return then_const


def register_transformation(
    constructor: Callable[..., Transformation]
) -> Callable[..., PartialConstructor]:
    return _register("transformations", constructor)


def register_measurement(
    constructor: Callable[..., Measurement]
) -> Callable[..., PartialConstructor]:
    return _register("measurements", constructor)


def register_combinator(
    constructor: Callable[..., Union[Transformation, Measurement]],
) -> Callable[..., PartialConstructor]:
    return _register("combinators", constructor)  # pragma: no cover


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
