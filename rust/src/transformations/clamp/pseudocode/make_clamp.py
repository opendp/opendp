# type: ignore
def make_clamp(
    input_domain: VectorDomain[AtomDomain[TA]], 
    input_metric: M, 
    bounds: tuple[TA, TA]
): # |\label{line:def}|
    input_domain.element_domain.assert_non_null() # |\label{line:assert-non-null}|

    # clone to make it explicit that we are not mutating the input domain
    output_row_domain = input_domain.element_domain.clone()
    output_row_domain.bounds = Bounds.new_closed(bounds)

    def clamper(value: TA) -> TA: # |\label{line:clamper}|
        return value.total_clamp(bounds[0], bounds[1])
    
    return make_row_by_row_fallible( # |\label{line:row-by-row}|
        input_domain, 
        input_metric, 
        output_row_domain, 
        clamper
    )