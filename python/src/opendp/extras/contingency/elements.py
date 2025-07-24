from typing import Any
from opendp._internal import _extrinsic_distance, _extrinsic_domain
from opendp.mod import Domain, ExtrinsicDistance, ExtrinsicDomain, Metric


def dict_domain(element_domains: dict[Any, Domain]) -> ExtrinsicDomain:
    """Domain containing a fixed-key dictionary of elements, each with its own domain."""

    def _member(x):
        if not isinstance(x, dict):
            raise ValueError("Expected a dict")
        return all(domain_i.member(x_i) for domain_i, x_i in zip(element_domains, x))

    ident = ", ".join(f"{k}: {str(d)}" for k, d in element_domains.items())
    return _extrinsic_domain(
        identifier=f"DictDomain({ident})",
        member=_member,
        descriptor=DictDescriptor(element_domains),
    )


class DictDescriptor(dict):
    pass


def linf_norm(inner_metric: Metric) -> ExtrinsicDistance:
    """Linf norm of inner metric distances.

    The metric forms a valid metric space when paired with a collection domain
    where each element value is an `arrayd_domain` of numbers.

    The metric computes the pairwise L2 distance between each value in the dictionary,
    and then takes the maximum of these distances.

    That is, the sensitivity of a dictionary of d-dimensional arrays
    is the maximum L2 sensitivity of any array in the dictionary.
    """
    return _extrinsic_distance(
        identifier=f"LInf({inner_metric})", descriptor=inner_metric
    )
