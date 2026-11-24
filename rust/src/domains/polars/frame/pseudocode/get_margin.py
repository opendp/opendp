# type: ignore
def get_margin(domain: FrameDomain, by: set[Expr]) -> Margin:
    margin = next(  # |\label{let-margin}|
        (m for m in domain.margins if m.by == by), Margin.by(by)
    )

    subset_margins = [  # |\label{let-subset-margins}|
        margin for margin in domain.margins if margin.by.issubset(by)
    ]

    margin.max_length = min(  # |\label{let-max-length}|
        (m.max_length for m in subset_margins if m.max_length is not None), default=None
    )

    all_max_groups = [  # |\label{all-max-groups}|
        (m.by, m.max_groups) for m in domain.margins if m.max_groups is not None
    ]
    max_groups_covering = find_min_covering(by.copy(), all_max_groups)
    margin.max_groups = None
    if max_groups_covering is not None:
        try:
            product = u32(1)
            for _, v in max_groups_covering:
                product = product.inf_mul(v)
            margin.max_groups = product
        except OpenDPException:
            pass

    all_invariants = (  # |\label{all-invariants}|
        m.invariant for m in domain.margins if by.issubset(m.by)
    )
    margin.invariant = max(
        all_invariants, key={None: 0, "Keys": 1, "Lengths": 2}.get, default=None
    )

    return margin
