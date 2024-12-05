# type: ignore
def make_row_by_row_fallible(
    input_domain: DI, 
    input_metric: M, 
    output_row_domain: DO, 
    # a function from input domain's row type to output domain's row type
    row_function: Callable([[DI_RowDomain_Carrier], DO_RowDomain_Carrier])
) -> Transformation:
    
    # where .translate is defined by the RowByRowDomain trait
    output_domain = input_domain.translate(output_row_domain)

    def function(data: DI_Carrier) -> DO_Carrier:
        # where .apply_rows is defined by the RowByRowDomain trait
        return DI.apply_rows(data, row_function)

    stability_map = new_stability_map_from_constant(1) # |\label{line:stability-map}|

    return Transformation(
        input_domain, output_domain, function,
        input_metric, input_metric, stability_map)