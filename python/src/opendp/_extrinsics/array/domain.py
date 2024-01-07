from opendp.mod import Domain


def np_array2_domain(
    *, norm=None, p=None, origin=None, size=None, num_columns=None, T=None
) -> Domain:
    """Construct a new Domain representing 2-dimensional numpy arrays.
    
    :param norm: each row in x is bounded by the norm
    :param p: designate which L_`ord` norm
    :param origin: center of the norm region. Assumed to be at zero
    :param size: number of rows in data
    :param num_columns: number of columns in the data
    :param T: atom type
    """
    import numpy as np # type: ignore[import]
    import opendp.prelude as dp

    if (norm is None) != (p is None):
        raise ValueError("norm and p must both be set")
    if norm is not None:
        if norm < 0:
            raise ValueError("norm must be non-negative")
        if p not in {1, 2}:
            raise ValueError("expected an order p of 1 or 2")

    # check that origin is well-formed    
    if origin is not None:
        if norm is None:
            raise ValueError("origin may only be set if data has bounded norm")

        if isinstance(origin, np.ndarray):
            if origin.ndim == 1:
                if num_columns is None:
                    num_columns = origin.size
                if num_columns != origin.size:
                    raise ValueError(f"origin must have {num_columns} values")
            if origin.ndim not in {0, 1}:
                raise ValueError("origin must have 0 or 1 dimensions")
        elif not isinstance(origin, (int, float)):
            raise ValueError("origin must be a number")
        
    if num_columns is not None:
        if isinstance(num_columns, int) and num_columns < 0:
            raise ValueError("num_columns must be non-negative")

    desc = {
        "origin": origin,
        "norm": norm,
        "p": p,
        "size": size,
        "num_columns": num_columns,
        "T": dp.parse_or_infer(T, norm),
    }
    desc = {k: v for k, v in desc.items() if v is not None}

    def member(x):
        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        if x.ndim != 2:
            raise ValueError("Expected 2-dimensional array")
        if num_columns is not None and x.shape[0] != num_columns:
            raise ValueError(f"must have {num_columns} columns")
        if origin is not None:
            x = x - origin
        if norm is not None:
            max_norm = np.linalg.norm(x, ord=p, axis=1).max()
            if max_norm > norm:
                raise ValueError(f"row norm is too large. {max_norm} > {norm}")
        if size is not None and len(x) != size:
            raise ValueError(f"expected exactly {size} rows")
        return True

    ident = f"NPArray2Domain({', '.join(f'{k} = {v}' for k, v in desc.items())})"

    return dp.user_domain(ident, member, desc)


def _np_xTx_domain(*, num_features, norm=None, p=None, size=None, T) -> Domain:
    """The domain of square symmetric matrices formed by computing x^Tx,
    for some dataset x.

    :param num_features: number of rows/columns in the matrix
    :param norm: each row in x is bounded by the norm
    :param p: designate which L_`ord` norm
    :param size: number of rows in data
    """
    desc = locals()
    import opendp.prelude as dp

    desc["T"] = dp.RuntimeType.parse(T)

    if num_features is not None:
        if isinstance(num_features, int) and num_features < 0:
            raise ValueError("num_features must be non-negative")

    def member(x):
        import numpy as np # type: ignore[import]

        if not isinstance(x, np.ndarray):
            raise TypeError("must be a numpy ndarray")
        if x.shape != (num_features,) * 2:
            raise ValueError(f"expected a square array with {num_features} features")
        return True

    ident = f"NPCovDomain({', '.join(f'{k} = {v}' for k, v in desc.items())})"

    return dp.user_domain(ident, member, desc)
