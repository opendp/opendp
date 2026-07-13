from __future__ import annotations

import pytest

from opendp.extras.sklearn.cluster import BinaryPEMeans, BinaryPEMeansConfig

np = pytest.importorskip("numpy")
sparse = pytest.importorskip("scipy.sparse")


def test_binary_pe_means_fits_toy_sparse_data():
    x = sparse.csr_matrix(
        np.array(
            [
                [1, 1, 0, 0, 0, 0],
                [1, 0, 1, 0, 0, 0],
                [0, 0, 0, 1, 1, 0],
                [0, 0, 0, 1, 0, 1],
            ],
            dtype=np.float32,
        )
    )

    model = BinaryPEMeans(
        n_features=x.shape[1],
        n_clusters=2,
        epsilon=1.0,
        delta=1e-6,
        random_state=7,
        config=BinaryPEMeansConfig(
            iterations=3,
            population_size=16,
            init_from_data_sample=False,
            noise_sigma=0.25,
            batch_size=8,
            center_active_tags=2,
            store_internal_labels=True,
        ),
    )
    fitted = model.fit(x)

    assert fitted is model
    assert model.cluster_centers_ is not None
    assert model.labels_ is not None
    assert model.cluster_centers_.shape == (2, x.shape[1])
    assert model.labels_.shape == (x.shape[0],)
    assert np.isfinite(model.score(x))


def test_binary_pe_means_logs_rho_and_threshold_metadata():
    x = sparse.csr_matrix(
        np.array(
            [
                [1, 0, 0, 1, 0, 0],
                [1, 1, 0, 0, 0, 0],
                [0, 0, 1, 0, 1, 0],
                [0, 0, 1, 0, 0, 1],
            ],
            dtype=np.float32,
        )
    )

    model = BinaryPEMeans(
        n_features=x.shape[1],
        n_clusters=2,
        rho=0.5,
        delta=1e-6,
        random_state=11,
        config=BinaryPEMeansConfig(
            iterations=2,
            population_size=8,
            init_from_data_sample=False,
            batch_size=8,
            center_active_tags=2,
            store_internal_labels=False,
        ),
    )
    model.fit(x)

    assert model.extra_["rho_total"] == pytest.approx(0.5)
    assert model.extra_["rho_step"] == pytest.approx(0.25)
    assert model.extra_["candidate_weight_threshold"] == pytest.approx(model.extra_["vote_noise_scale"])
