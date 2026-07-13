from __future__ import annotations

from typing import Dict

from .base import ClusterAlgorithm, SklearnKMeans, SklearnKMedians
from .google_lsh import GoogleLSHCoresetKMeans
from .hst_google import GoogleHSTClustering
from .opendp_pe import OpenDPSparsePEMeans, OpenDPSparsePEMeansConfig
from .opendp_pr2664 import OpenDPKMeans, OpenDPKMedians
from .pe_means import PEMeans, PEMeansConfig


def build_algorithms(args) -> Dict[str, ClusterAlgorithm]:
    pe_cfg = PEMeansConfig(
        iterations=args.pe_iterations,
        population_size=args.pe_population,
        mutation_scale=args.pe_mutation_scale,
        levy_alpha=args.pe_levy_alpha,
        noise_sigma=args.pe_noise_sigma,
        adaptive_population=not args.pe_no_adaptive_population,
    )
    opendp_pe_cfg = OpenDPSparsePEMeansConfig(
        iterations=getattr(args, "pe_iterations", 16),
        population_size=getattr(args, "pe_population", 512),
        center_active_tags=getattr(args, "pe_center_active_tags", 96),
        min_active_tags=getattr(args, "pe_min_active_tags", 16),
        max_active_tags=getattr(args, "pe_max_active_tags", 160),
        mutation_drop_prob=getattr(args, "pe_mutation_drop_prob", 0.18),
        mutation_add_mean=getattr(args, "pe_mutation_add_mean", 18.0),
        distance=getattr(args, "pe_distance", "weighted_jaccard"),
        batch_size=getattr(args, "pe_batch_size", 8192),
        backend=getattr(args, "pe_backend", "auto"),
        neighboring=getattr(args, "pe_neighboring", "add_remove"),
        noise_sigma=getattr(args, "pe_noise_sigma", None),
        noisy_candidate_weight_threshold_multiplier=getattr(
            args,
            "pe_noisy_candidate_weight_threshold_multiplier",
            1.0,
        ),
    )
    return {
        "sklearn": SklearnKMeans(n_init=args.sklearn_n_init),
        "sklearn_kmedians_like": SklearnKMedians(),
        "opendp_kmeans": OpenDPKMeans(scale=args.opendp_scale, max_depth=args.opendp_max_depth),
        "opendp_kmedians": OpenDPKMedians(scale=args.opendp_scale, max_depth=args.opendp_max_depth),
        "google_lsh": GoogleLSHCoresetKMeans(max_depth=args.google_lsh_max_depth),
        "hst": GoogleHSTClustering(layers=args.hst_layers, num_buckets_beam=args.hst_num_buckets_beam),
        "pe_means": PEMeans(pe_cfg),
        "opendp_pe_means": OpenDPSparsePEMeans(
            epsilon=getattr(args, "epsilon", 1.0),
            delta=getattr(args, "delta", 1e-6),
            random_state=getattr(args, "seed", None),
            config=opendp_pe_cfg,
        ),
    }
