# type: ignore
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

    margin.max_length = min(  # |\label{let-ml}|
        m.max_length
        for m in subset_margins
        if m.max_length is not None
    )

    from math import prod

    all_mngs = [  # |\label{all-mngs}|
        (m.by, m.max_groups)
        for m in domain.margins
        if m.max_groups is not None
    ]
    mngs_covering = find_min_covering(grouping_columns, all_mngs)
    if mngs_covering is not None:
        margin.max_num_partitions = prod(v for _, v in mngs_covering)

    all_infos = (  # |\label{all-infos}|
        m.invariant
        for m in domain.margins
        if by.issubset(m.by)
    )
    margin.invariant = max(all_infos, key={None: 0, "Keys": 1, "Lengths": 2}.get)

    if not by:
        if margin.invariant is None:
            margin.invariant = Invariant.Keys
        margin.max_groups = 1
    
    return margin
