# type: ignore 
def make_row_by_row( 
    input_domain: DI,  
    input_metric: M,  
    output_row_domain: DO_RowDomain,  
    # a function from input domain's row type to output domain's row type 
    row_function: Callable([[DI_RowDomain_Carrier], DO_RowDomain_Carrier]) 
) -> Transformation: 
    # Python doesn't model errors in the type system,  
    # so in the Rust code the return value from the function is wrapped in Ok 
    return make_row_by_row_fallible( 
        input_domain, input_metric, output_row_domain, row_function 
    ) 