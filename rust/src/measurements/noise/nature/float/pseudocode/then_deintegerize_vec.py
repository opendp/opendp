# type: ignore
def then_deintegerize_vec(k: i32) -> Function[Vec[IBig], Vec[TO]]:

    if k == i32.MIN: # |\label{line:check-k}|
        raise ValueError("k must be greater than i32.MIN")
    
    def element_function(x_i):
        return TO.from_rational(x_mul_2k(RBig.from_(x_i), k))
    
    return Function.new(lambda x: [element_function(x_i) for x_i in x])
