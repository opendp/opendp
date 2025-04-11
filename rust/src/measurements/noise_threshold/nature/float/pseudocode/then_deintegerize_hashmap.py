# type: ignore
def then_deintegerize_hashmap(k: i32) -> Function[HashMap[TK, IBig], HashMap[TK, TV]]:
    if k == i32.MIN: # |\label{line:check-k}|
        raise ValueError("k must not be i32.MIN")

    def value_function(v_i):
        return TV.from_rational(x_mul_2k(RBig.from_(v_i), k))

    return Function.new(lambda x: {k_i: value_function(v_i) for k_i, v_i in x.items()})
