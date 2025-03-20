# type: ignore
def then_deintegerize_vec() -> Function[Vec[IBig], Vec[T]]:
    def element_function(x_i):
        return TO.from_rational(x_mul_2k(x_i, k))
    
    return Function.new(lambda x: [element_function(x_i) for x_i in x])
