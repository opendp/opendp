from opendp._internal import _extrinsic_distance, _extrinsic_domain
from opendp.mod import Domain


def tuple_domain(element_domains: tuple[Domain, ...]):
    """Domain containing a fixed-length tuple of elements, each with its own domain."""

    def _member(x):
        if not isinstance(x, list):
            raise ValueError("Expected a tuple")
        return all(domain_i.member(x_i) for domain_i, x_i in zip(element_domains, x))

    return _extrinsic_domain(
        identifier=f"TupleDomain({', '.join(str(d) for d in element_domains)})",
        member=_member,
        descriptor=element_domains,
    )


def linf_l2_distance():
    """Linf L2 distance metric.

    The metric forms a valid metric space when paired with a `tuple_domain`
    where each element_domain is an `arrayd_domain` of numbers.

    The metric computes the pairwise L2 distance between each element in the list,
    and then takes the maximum of these distances.

    That is, the sensitivity of a vector of d-dimensional arrays
    is the maximum L2 sensitivity of any array in the list.
    """
    return _extrinsic_distance(descriptor="LInfL2Distance")

