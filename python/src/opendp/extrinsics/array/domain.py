import opendp.prelude as dp


def np_array2_domain(
    *, origin=None, norm=None, ord=None, size=None, num_columns=None, T=None
) -> dp.Domain:
    """Construct a new Domain representing 2-dimensional numpy arrays."""
    desc = locals()
    desc["T"] = dp.parse_or_infer(T, norm)
    desc = {k: v for k, v in desc.items() if v is not None}

    if norm is not None and ord not in {1, 2}:
        raise ValueError("expected an ord of 1 or 2")
    if num_columns is not None:
        if isinstance(num_columns, int) and num_columns < 0:
            raise ValueError("num_columns must be non-negative")

    def member(x):
        import numpy as np

        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        if x.ndim != 2:
            raise ValueError(f"Expected 2-dimensional array")
        if origin is not None:
            x = x - origin
        if norm is not None and max(np.linalg.norm(x, ord=ord, axis=1)) > norm:
            raise ValueError(f"there exist some row(s) with norm greater than {norm}")
        if size is not None and len(x) != size:
            raise ValueError(f"expected exactly {size} rows")
        return True

    ident = f"NPArray2Domain({', '.join(f'{k} = {v}' for k, v in desc.items())})"

    return dp.user_domain(desc, member, ID=ident)


def np_cov_domain(*, norm=None, ord=None, num_features, size=None, T):
    desc = locals()
    desc["T"] = dp.RuntimeType.parse(T)

    if num_features is not None:
        if isinstance(num_features, int) and num_features < 0:
            raise ValueError("num_features must be non-negative")

    def member(x):
        import numpy as np

        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        if x.ndim != 2:
            raise ValueError(f"Expected 2-dimensional array")
        return True

    ident = f"NPCovDomain({', '.join(f'{k} = {v}' for k, v in desc.items())})"

    return dp.user_domain(desc, member, ID=ident)
