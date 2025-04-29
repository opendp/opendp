# type: ignore
def get_rounding_distance() -> RBig:
    k_min = get_min_k(T) # |\label{line:get-min-k}|
    if k < k_min: # |\label{line:check-k}|
        raise f"k ({k}) must not be smaller than {k_min}"

    # input has granularity 2^{k_min} (subnormal float precision)
    input_gran = x_mul_2k(RBig.ONE, k_min) # |\label{line:input-gran}|

    # discretization rounds to the nearest 2^k
    output_gran = x_mul_2k(RBig.ONE, k) # |\label{line:output-gran}|

    # the worst-case increase in sensitivity due to discretization is
    #     the range, minus the smallest step in the range
    distance = output_gran - input_gran # |\label{line:distance}|

    # rounding may occur on all vector elements
    if not distance.is_zero(): # |\label{line:zero-distance}|
        if size is None: # |\label{line:unknown-size}|
            raise "domain size must be known if discretization is not exact"
        
        match P:
            case 1:
                distance *= RBig.from_(size)
            case 2:
                distance *= RBig.try_from(f64.inf_cast(size).inf_sqrt())
            case _:
                raise f"norm ({P}) must be one or two"
        
    return distance