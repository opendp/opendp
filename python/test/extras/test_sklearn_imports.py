from pathlib import Path

import pytest


def test_sgd_classifier_lazy_export():
    pytest.importorskip("sklearn")
    from opendp.extras.sklearn.linear_model import SGDClassifier
    from opendp.extras.sklearn.linear_model._sgd_classifier import (
        SGDClassifier as implementation,
    )

    assert SGDClassifier is implementation


def test_sklearn_hbc_fallback_is_only_for_unreviewed_rowwise_inference():
    # Global pytest features are separate from requirements of public APIs.
    sklearn = (
        Path(__file__).resolve().parents[2] / "src" / "opendp" / "extras" / "sklearn"
    )
    violations = []
    for source in sklearn.rglob("*.py"):
        content = source.read_text(encoding="utf-8")
        for constructor in ("make_user_transformation", "make_user_measurement"):
            if constructor in content:
                violations.append(f"{source.relative_to(sklearn)}: {constructor}")
        if source.name != "_rowwise.py" and "honest-but-curious" in content:
            violations.append(
                f"{source.relative_to(sklearn)}: unexpected HBC requirement"
            )
    rowwise = (sklearn / "_rowwise.py").read_text(encoding="utf-8")
    expected = (
        "if not _trusted_rowwise(fitted):\n"
        '        assert_features("honest-but-curious")'
    )
    assert expected in rowwise
    assert not violations, ", ".join(violations)
