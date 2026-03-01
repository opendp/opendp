# type: ignore
from math import prod

def get_margin(domain: FrameDomain, by: set[Expr]) -> Margin:
    margin = next(  # |\label{let-margin}|
        (m for m in domain.margins if m.by == by), 
        Margin.by(by)
    )

    subset_margins = [  # |\label{let-subset-margins}|
        margin
        for margin in domain.margins
        if margin.by.issubset(by)
    ]

    margin.max_length = min(  # |\label{let-max-length}|
        m.max_length
        for m in subset_margins
        if m.max_length is not None
    )

    all_max_groups = [  # |\label{all-max-groups}|
        (m.by, m.max_groups)
        for m in domain.margins
        if m.max_groups is not None
    ]
    max_groups_covering = find_min_covering(grouping_columns, all_max_groups)
    if max_groups_covering is not None:
        margin.max_groups = prod(v for _, v in max_groups_covering)

    all_invariants = (  # |\label{all-invariants}|
        m.invariant
        for m in domain.margins
        if by.issubset(m.by)
    )
    margin.invariant = max(all_invariants, key={None: 0, "Keys": 1, "Lengths": 2}.get)

    return margin
