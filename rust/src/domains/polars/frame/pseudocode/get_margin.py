# type: ignore
def get_margin(domain: FrameDomain, grouping_columns: set[str]) -> Margin:
    margin = domain.margins.get(
        grouping_columns, Margin.default()
    )  # |\label{let-margin}|

    subset_margins = {  # |\label{let-subset-margins}|
        by: margin
        for by, margin in domain.margins.items()
        if by.issubset(grouping_columns)
    }

    margin.max_partition_length = min(  # |\label{let-mpl}|
        m.max_partition_length
        for m in subset_margins.values()
        if m.max_partition_length is not None
    )

    margin.max_partition_contributions = min(  # |\label{let-mpc}|
        m.max_partition_contributions
        for m in subset_margins.values()
        if m.max_partition_contributions is not None
    )

    from math import prod

    all_mnps = {  # |\label{all-mnps}|
        by: m.max_num_partitions
        for by, m in domain.margins.items()
        if m.max_num_partitions is not None
    }
    mnps_covering = find_min_covering(grouping_columns, all_mnps)
    if mnps_covering is not None:
        margin.max_num_partitions = prod(mnps_covering.values())

    all_mips = {  # |\label{all-mips}|
        by: m.max_influenced_partitions
        for by, m in domain.margins.items()
        if m.max_influenced_partitions is not None
    }
    mips_covering = find_min_covering(grouping_columns, all_mips)
    if mips_covering is not None:
        margin.max_influenced_partitions = prod(mips_covering.values())

    all_infos = (  # |\label{all-infos}|
        m.public_info
        for by, m in domain.margins.items()
        if grouping_columns.issubset(by)
    )
    margin.public_info = max(all_infos, key={None: 0, "Keys": 1, "Lengths": 2}.get)

    return margin
