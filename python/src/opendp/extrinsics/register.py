from typing import Any, Callable, Optional, Union
from opendp.mod import PartialConstructor, Measurement, Transformation, Domain, Metric


def _to_then(func):
    """Convert any `make_` constructor to a `then_` constructor"""

    def then_func(*args, **kwargs):
        return PartialConstructor(
            lambda input_domain, input_metric: func(
                input_domain, input_metric, *args, **kwargs
            )
        )

    return then_func


def _register(module, constructor):
    import importlib, inspect

    module = importlib.import_module(f"opendp.{module}")
    module.__dict__[constructor.__name__] = constructor
    arg_names = inspect.getfullargspec(constructor)[0]

    if len(arg_names) >= 2 and arg_names[:2] == ["input_domain", "input_metric"]:
        then_name = constructor.__name__.replace("make_", "then_", 1)
        then_const = _to_then(constructor)
        module.__dict__[then_name] = then_const
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
