"""Import regressions must also run when sklearn is not installed."""

import subprocess
import sys

import pytest


def test_other_opendp_apis_import_without_sklearn():
    script = """
import importlib.abc
import sys
import pytest
class NoSklearn(importlib.abc.MetaPathFinder):
    def find_spec(self, fullname, path=None, target=None):
        if fullname == "sklearn" or fullname.startswith("sklearn."):
            raise ModuleNotFoundError("sklearn is unavailable", name=fullname)
sys.meta_path.insert(0, NoSklearn())
import opendp.prelude as dp
assert dp.symmetric_distance() is not None
dp.enable_features("contrib")
data = dp.sklearn.SklearnFit([1, 2], [0, 1])
assert dp.domain_of(data, infer=True).member(data)
for name in ("decomposition", "linear_model"):
    with pytest.raises(ImportError, match="optional install scikit-learn"):
        getattr(dp.sklearn, name)
with pytest.raises(ModuleNotFoundError):
    importlib.import_module("opendp.extras.sklearn._estimator")
# Importing a low-level constructor first imports its parent package. Neither
# this path nor autodoc of linear_model should require the sklearn estimator.
from opendp.extras.sklearn.linear_model._make_private_theil_sen import make_private_theil_sen
from opendp.extras.sklearn.linear_model._make_private_logistic_regression import make_private_logistic_regression
from opendp.extras.sklearn import linear_model
assert callable(make_private_theil_sen)
assert callable(make_private_logistic_regression)
assert linear_model.LinearRegression is not None
assert "LogisticRegression" in dir(linear_model)
assert "opendp.extras.sklearn._estimator" not in sys.modules
with pytest.raises(ImportError, match="optional install scikit-learn"):
    linear_model.LogisticRegression
with pytest.raises(AttributeError):
    linear_model.unknown_estimator
"""
    subprocess.run([sys.executable, "-c", script], check=True)


def test_logistic_regression_lazy_export():
    pytest.importorskip("sklearn")
    from sklearn.base import clone
    from opendp.extras.sklearn.linear_model import LogisticRegression
    from opendp.extras.sklearn.linear_model._logistic_regression import (
        LogisticRegression as implementation,
    )

    assert LogisticRegression is implementation
    module = sys.modules["opendp.extras.sklearn.linear_model"]
    assert "LogisticRegression" in dir(module)
    assert module.__getattr__("LogisticRegression") is implementation
    model = LogisticRegression(n_iters=3)
    assert clone(model).get_params() == model.get_params()
