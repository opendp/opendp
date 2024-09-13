# type: ignore
def make_is_equal(
    input_domain: VectorDomain[AtomDomain[TIA]], 
    input_metric: M, 
    value: TIA
): # |\label{line:def}|
    output_row_domain = atom_domain(T=bool)

    def is_equal(arg: TA) -> TA: # |\label{line:function}|
        return value == arg

    return make_row_by_row( # |\label{line:row-by-row}|
        input_domain, 
        input_metric, 
        output_row_domain, 
        is_equal
    )